use crate::images::extract_images_from_docx;
use crate::latex_ext::LatexStringExt;
use crate::pandoc_ext::run_pandoc;
use crate::util::copy_recursively;

use std::error::Error;
use std::fs::{copy, create_dir_all, read_to_string, write};
use std::io;
use std::path::{Path, PathBuf};

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub fn create_project<P: AsRef<Path>>(project_dir: P) -> Result<(), Box<dyn Error>> {
    let project_dir = project_dir.as_ref();

    // Copy template into the new project directory
    let template_path = Path::new("/home/omega/projects/vestnik/newman/template");
    copy_recursively(template_path, project_dir)?;
    Ok(())
}

pub fn update_project<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    project_dir: Q,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let project_dir = project_dir.as_ref();

    // Derive part name from input filename
    let part_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Failed to extract part name from input path")?;

    // Create directory for article .tex files
    let part_dir = Path::new(project_dir).join("src").join(part_name);
    create_dir_all(&part_dir)?;
    copy(input_path, part_dir.join(input_path.file_name().unwrap()))?;

    // Create media directory
    let media_dir = Path::new(project_dir).join("media").join(part_name);
    create_dir_all(&media_dir)?;

    let main_path = Path::new(project_dir).join("main.tex");

    // Run Pandoc and process text
    let mut text = run_pandoc(input_path);
    text.replace_textbf();
    text.remove_short_bfseries()?;
    text.fix_enumerate()?;
    text.fix_itemize()?;
    text.fix_number_spacing()?;
    text.remove_tag("ul");
    text.remove_tag("hl");
    text.remove_tag("pandocbounded");
    text.comment_out_tables();
    text.change_latex_quotes();
    text.replace_envelopes();
    text.remove_tightlists();
    text.unindent();
    text.replace_bullets();
    text.fix_images(part_name);

    // Split into individual articles
    let articles: Vec<String> = text.split_articles();

    // Write each article into its own numbered .tex file
    for (i, article) in articles.iter().enumerate() {
        let file_path = part_dir.join(format!("{:03}.tex", i + 1));
        write(file_path, article)?;
    }

    // Extract images
    extract_images_from_docx(input_path.to_str().unwrap(), media_dir.to_str().unwrap())?;

    let articles_main: String = (0..articles.len())
        .map(|i| format!("\n\\input{{src/{part_name}/{:03}.tex}}", i + 1))
        .collect();
    let contents = read_to_string(&main_path)?;

    let mut new_content = String::new();
    let mut inserted = false;

    for line in contents.lines() {
        new_content.push_str(line);
        new_content.push('\n');
        if !inserted && line.trim() == "% Main content" {
            new_content.push_str(&articles_main);
            inserted = true;
        }
    }

    write(&main_path, new_content)?;
    Ok(())
}

pub fn watch_and_compile_project(project_dir: Option<PathBuf>) -> io::Result<()> {
    let project_dir = project_dir.unwrap_or_else(|| PathBuf::from("."));
    let build_dir = project_dir.join("build");
    create_dir_all(&build_dir)?;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    watcher
        .watch(&project_dir, RecursiveMode::Recursive)
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
                    println!("🔁 Detected change: {}", path.display());
                }

                triggered = true;
                last_event = Instant::now();
            }
            Ok(Err(e)) => eprintln!("⚠️ Watch error: {e}"),
            Err(_) => {
                if triggered && last_event.elapsed() >= debounce_duration {
                    triggered = false;

                    println!("✅ Compilation started...");
                    let status = std::process::Command::new("tectonic")
                        .args(&["-X", "compile", "main.tex"])
                        .arg(format!("--outdir={}", build_dir.display()))
                        .current_dir(&project_dir)
                        .status();

                    match status {
                        Ok(status) if status.success() => println!("✅ Compilation succeeded!"),
                        Ok(status) => eprintln!("❌ Compilation failed with status: {}", status),
                        Err(e) => eprintln!("❌ Failed to run tectonic: {}", e),
                    }
                }
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
