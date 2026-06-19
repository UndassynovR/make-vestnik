use crate::images::extract_images_from_docx;
use crate::latex_ext::LatexStringExt;
use crate::pandoc_ext::run_pandoc;
use crate::util::copy_recursively;
use colored::*;

use std::env;
use std::error::Error;
use std::fs::{self, copy, create_dir_all, read_to_string, write};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub struct Project {
    pub root_dir: PathBuf,
    src_dir: PathBuf,
    media_dir: PathBuf,
    build_dir: PathBuf,
    main_tex: PathBuf,
}

impl Project {
    /// Create a new Project instance from a directory path
    pub fn new<P: AsRef<Path>>(project_dir: P) -> Result<Self, Box<dyn Error>> {
        let root_dir = project_dir.as_ref().to_path_buf();

        Ok(Project {
            src_dir: root_dir.join("src"),
            media_dir: root_dir.join("media"),
            build_dir: root_dir.join("build"),
            main_tex: root_dir.join("main.tex"),
            root_dir,
        })
    }

    /// Initialize a new project by copying template files
    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        let template_path = if cfg!(debug_assertions) {
            // Debug: relative to project root
            let exe_path = env::current_exe()?;
            let exe_dir = exe_path.parent().unwrap();
            let project_root = exe_dir
                .parent()
                .and_then(|p| p.parent())
                .ok_or("Could not find project root")?;
            project_root.join("template")
        } else {
            // Release: installed templates in $out/share/make-vestnik/templates
            let exe_path = env::current_exe()?;
            let exe_dir = exe_path.parent().unwrap();
            exe_dir
                .parent() // $out/bin/.. = $out
                .ok_or("Could not find $out root")?
                .join("share/make-vestnik/templates")
        };

        if !template_path.exists() {
            return Err(format!("Template directory not found: {:?}", template_path).into());
        }

        copy_recursively(&template_path, &self.root_dir)?;

        // FIX: Make main.tex writable after copying from template
        if self.main_tex.exists() {
            let mut perms = fs::metadata(&self.main_tex)?.permissions();
            perms.set_readonly(false);
            fs::set_permissions(&self.main_tex, perms)?;
        }

        Ok(())
    }

    /// Add a DOCX file to the project, converting it to LaTeX
    pub fn add<P: AsRef<Path>>(&self, input_path: P) -> Result<(), Box<dyn Error>> {
        let input_path = input_path.as_ref();

        // Derive part name from input filename
        let part_name = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Failed to extract part name from input path")?;

        println!("Processing: {}", part_name);

        // Create directory for article .tex files
        let part_dir = self.src_dir.join(part_name);
        create_dir_all(&part_dir)?;
        copy(input_path, part_dir.join(input_path.file_name().unwrap()))?;

        // Create media directory
        let media_dir = self.media_dir.join(part_name);
        create_dir_all(&media_dir)?;

        // Extract images FIRST before any text processing
        println!("Extracting images from DOCX...");
        extract_images_from_docx(input_path, &media_dir)?;

        // Run Pandoc and process text
        println!("Converting to LaTeX...");
        let mut text = run_pandoc(input_path);
        text.remove_tag("label");
        text.unwrap_tag("section");
        text.replace_textbf();
        text.remove_short_bfseries()?;
        text.fix_lists();
        text.fix_number_spacing()?;
        text.remove_tag("ul");
        text.unwrap_tag("hl");
        text.unwrap_tag("RL");
        text.remove_tag("pandocbounded");
        text.comment_out_tables();
        text.change_latex_quotes();
        text.replace_envelopes();
        text.remove_tightlists();
        text.unindent();
        text.replace_bullets();
        text.fix_images(part_name);
        text.replace_super_sub_scripts();
        text.fix_email_links();
        text.remove_zero_hspace();
        text.replace_textless();

        // Split into individual articles
        println!("Splitting into articles...");
        let articles: Vec<String> = text.split_articles();
        println!("Found {} articles", articles.len());

        // Write each article into its own numbered .tex file
        for (i, article) in articles.iter().enumerate() {
            let file_path = part_dir.join(format!("{:03}.tex", i + 1));
            write(&file_path, article)?;
            println!("Written: {}", file_path.display());
        }

        // Update main.tex with new article inputs
        println!("Updating main.tex...");
        let articles_main: String = (0..articles.len())
            .map(|i| format!("\n\\input{{src/{part_name}/{:03}.tex}}", i + 1))
            .collect();

        println!("main.tex path: {}", self.main_tex.display());

        // Check if main.tex exists and is writable
        if !self.main_tex.exists() {
            return Err(format!("main.tex not found at: {}", self.main_tex.display()).into());
        }

        let contents = read_to_string(&self.main_tex)
            .map_err(|e| format!("Failed to read main.tex: {}", e))?;

        let mut new_content = String::new();
        let mut inserted = false;

        for line in contents.lines() {
            new_content.push_str(line);
            new_content.push('\n');
            if !inserted && line.trim() == "% Main content" {
                new_content.push_str(&articles_main);
                inserted = true;
                println!("Inserted {} article references", articles.len());
            }
        }

        if !inserted {
            println!("Warning: '% Main content' marker not found in main.tex");
        }

        write(&self.main_tex, new_content).map_err(|e| {
            format!(
                "Failed to write to main.tex: {} (Permission denied - check file permissions)",
                e
            )
        })?;

        println!("✓ Successfully added {}", part_name);
        Ok(())
    }

    /// Compile the project in watch mode (watches for file changes and auto-recompiles)
    pub fn compile(&self) -> io::Result<()> {
        create_dir_all(&self.build_dir)?;

        // Build once on startup
        println!("{}", "Initial compilation started...".cyan());
        self.build();

        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        watcher
            .watch(&self.root_dir, RecursiveMode::Recursive)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut last_event = Instant::now();
        let debounce_duration = Duration::from_millis(500);
        let mut triggered = false;

        loop {
            // Wait for an event with timeout
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if !matches!(event.kind, EventKind::Modify(_)) {
                        continue;
                    }
                    let relevant_paths = event
                        .paths
                        .into_iter()
                        .filter(|p| p.exists() && !should_ignore(p))
                        .collect::<Vec<_>>();

                    if relevant_paths.is_empty() {
                        continue;
                    }

                    for path in &relevant_paths {
                        println!(
                            "{}",
                            format!("Detected change: {}", path.display()).yellow()
                        );
                    }
                    triggered = true;
                    last_event = Instant::now();
                }
                Ok(Err(e)) => eprintln!("{}", format!("Watch error: {}", e).yellow()),
                Err(_) => {
                    if triggered && last_event.elapsed() >= debounce_duration {
                        triggered = false;
                        println!("{}", "Compilation started...".cyan());
                        self.build();
                    }
                }
            }
        }
    }

    /// Build the project once without watching (for one-time compilation)
    pub fn build_once(&self) -> io::Result<()> {
        create_dir_all(&self.build_dir)?;
        self.build();
        Ok(())
    }

    /// Show project status and information
    pub fn status(&self) -> Result<(), Box<dyn Error>> {
        println!("Project: {}", self.root_dir.display());
        println!("  Source directory: {}", self.src_dir.display());
        println!("  Media directory: {}", self.media_dir.display());
        println!("  Build directory: {}", self.build_dir.display());
        println!("  Main file: {}", self.main_tex.display());

        if self.main_tex.exists() {
            println!("  Status: {}", "Initialized".green());
        } else {
            println!("  Status: {}", "Not initialized".yellow());
        }

        if self.build_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.build_dir) {
                let pdf_count = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pdf"))
                    .count();
                if pdf_count > 0 {
                    println!("  PDFs built: {}", pdf_count);
                }
            }
        }

        Ok(())
    }

    /// Clean build artifacts
    pub fn clean(&self) -> Result<(), Box<dyn Error>> {
        if self.build_dir.exists() {
            std::fs::remove_dir_all(&self.build_dir)?;
            println!("Removed build directory: {}", self.build_dir.display());
        } else {
            println!("Build directory does not exist, nothing to clean.");
        }
        Ok(())
    }

    /// Open the PDF file in the default viewer
    pub fn open(&self) -> Result<(), Box<dyn Error>> {
        let pdf_path = self.build_dir.join("main.pdf");

        if !pdf_path.exists() {
            return Err(format!("PDF not found at: {}", pdf_path.display()).into());
        }

        println!("Opening PDF: {}", pdf_path.display());

        Command::new("xdg-open")
            .arg(&pdf_path)
            .spawn()
            .map_err(|e| format!("Failed to open PDF: {}", e))?;

        Ok(())
    }

    /// Perform a single build/compilation of the project
    fn build(&self) {
        let start_time = Instant::now();

        let status = std::process::Command::new("xelatex")
            .args(&[
                "-interaction=nonstopmode",
                "-halt-on-error",
                "-output-directory",
                &self.build_dir.display().to_string(),
                "main.tex",
            ])
            .current_dir(&self.root_dir)
            .status();

        let duration = start_time.elapsed();

        match status {
            Ok(status) if status.success() => {
                println!(
                    "{}",
                    format!("Compilation succeeded! ({:.2}s)", duration.as_secs_f64()).green()
                )
            }
            Ok(status) => {
                eprintln!(
                    "{}",
                    format!(
                        "Compilation failed with status: {} ({:.2}s)",
                        status,
                        duration.as_secs_f64()
                    )
                    .red()
                )
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    format!(
                        "Failed to run xelatex: {} ({:.2}s)",
                        e,
                        duration.as_secs_f64()
                    )
                    .red()
                )
            }
        }
    }
}

fn should_ignore(path: &Path) -> bool {
    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Ignore editor temp/undo/swap files
    if fname.starts_with(".#")
        || fname.ends_with("~")
        || fname.contains("undo-tree")
        || fname.ends_with(".swp")
        || fname.ends_with(".tmp")
    {
        return true;
    }

    // Ignore files in 'build' directory
    path.components().any(|c| c.as_os_str() == "build")
}
