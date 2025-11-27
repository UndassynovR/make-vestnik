mod images;
mod latex_ext;
mod pandoc_ext;
mod project;
mod util;

use project::Project;
use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_detailed_help();
        return Ok(());
    }

    let cmd = args[1].as_str();

    match cmd {
        "--help" | "-h" | "help" => {
            print_detailed_help();
            return Ok(());
        }
        "--version" | "-v" => {
            print_version();
            return Ok(());
        }
        _ => {}
    }

    // Check for --open flag
    let open_pdf = args.contains(&"--open".to_string());

    // Determine project directory (can be overridden by commands that accept it)
    let project_dir = get_project_dir(&args, cmd)?;
    let project = Project::new(&project_dir)?;

    match cmd {
        "init" => handle_init(&project)?,
        "add" => handle_add(&project, &args)?,
        "compile" | "watch" => handle_compile(&project, open_pdf)?,
        "build" => handle_build(&project, open_pdf)?,
        "status" => handle_status(&project)?,
        "clean" => handle_clean(&project)?,
        _ => {
            eprintln!(
                "Error: Unknown command '{}'. Use 'make-vestnik help' for usage.",
                cmd
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn get_project_dir(args: &[String], cmd: &str) -> Result<String, Box<dyn Error>> {
    // Filter out --open flag when looking for project_dir
    let filtered_args: Vec<String> = args.iter().filter(|a| *a != "--open").cloned().collect();

    // For 'add' command, project_dir might be at args[3]
    // For other commands, it's at args[2]
    let dir_index = if cmd == "add" && filtered_args.len() > 3 {
        3
    } else if filtered_args.len() > 2 && !filtered_args[2].ends_with(".docx") {
        2
    } else {
        return Ok(env::current_dir()?
            .to_str()
            .ok_or("Invalid current directory")?
            .to_string());
    };

    Ok(filtered_args[dir_index].clone())
}

fn print_detailed_help() {
    println!("make-vestnik - Document Project Manager\n");
    println!("USAGE:");
    println!("    make-vestnik <COMMAND> [OPTIONS]\n");
    println!("COMMANDS:");
    println!("    init [project_dir]");
    println!("        Initialize project (defaults to current directory)\n");
    println!("    add <docx_file> [project_dir]");
    println!("        Add or update a DOCX file in the project (defaults to current directory)\n");
    println!("    compile [project_dir] [--open]");
    println!("        Compile project in watch mode (auto-recompile on changes)\n");
    println!("    build [project_dir] [--open]");
    println!("        One-time compilation without watch mode\n");
    println!("    status [project_dir]");
    println!("        Show project information and compilation status\n");
    println!("    clean [project_dir]");
    println!("        Remove build artifacts and temporary files\n");
    println!("EXAMPLES:");
    println!("    make-vestnik init");
    println!("    make-vestnik init ./my-project");
    println!("    make-vestnik add document.docx");
    println!("    make-vestnik add document.docx ./my-project");
    println!("    make-vestnik compile --open");
    println!("    make-vestnik build ./my-project --open");
    println!("    make-vestnik status");
    println!("    make-vestnik clean\n");
    println!("OPTIONS:");
    println!("    --open         Open PDF after successful compilation");
    println!("    -h, --help     Show this help message");
    println!("    -v, --version  Show version information");
}

fn print_version() {
    println!("make-vestnik version {}", env!("CARGO_PKG_VERSION"));
}

fn handle_init(project: &Project) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(&project.root_dir);

    // Check if directory exists and if it's already initialized
    if path.exists() {
        println!("Initializing project in existing directory...");
    } else {
        println!("Creating and initializing new project...");
    }

    project.init()?;
    println!("✓ Project initialized successfully.");
    println!("\nNext steps:");
    println!("  1. Add a DOCX file: make-vestnik add <file.docx>");
    println!("  2. Start compiling: make-vestnik compile");

    Ok(())
}

fn handle_add(project: &Project, args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.len() < 3 {
        eprintln!("Error: add requires a DOCX file.");
        eprintln!("Usage: make-vestnik add <docx_file> [project_dir]");
        std::process::exit(1);
    }

    let docx_file = &args[2];

    if !docx_file.ends_with(".docx") {
        eprintln!("Error: File must be a .docx file.");
        std::process::exit(1);
    }

    if !PathBuf::from(docx_file).exists() {
        eprintln!("Error: DOCX file '{}' not found.", docx_file);
        std::process::exit(1);
    }

    if !PathBuf::from(&project.root_dir).exists() {
        eprintln!("Error: Project directory not found.");
        eprintln!("Use 'make-vestnik init' to create it first.");
        std::process::exit(1);
    }

    println!("Adding '{}'...", docx_file);
    project.add(docx_file)?;
    println!("✓ File added/updated successfully.");

    Ok(())
}

fn handle_compile(project: &Project, open_pdf: bool) -> Result<(), Box<dyn Error>> {
    if !PathBuf::from(&project.root_dir).exists() {
        eprintln!("Error: Project directory not found.");
        std::process::exit(1);
    }

    // Open PDF once before starting watch mode
    if open_pdf {
        if let Err(e) = project.open() {
            eprintln!("Warning: Failed to open PDF: {}", e);
        }
    }

    println!("Starting watch mode... Press Ctrl+C to stop.");
    project.compile()?;

    Ok(())
}

fn handle_build(project: &Project, open_pdf: bool) -> Result<(), Box<dyn Error>> {
    println!("Building project...");
    project.build_once()?;
    println!("✓ Build completed successfully.");

    // Open PDF after build completes
    if open_pdf {
        project.open()?;
    }

    Ok(())
}

fn handle_status(project: &Project) -> Result<(), Box<dyn Error>> {
    project.status()?;
    Ok(())
}

fn handle_clean(project: &Project) -> Result<(), Box<dyn Error>> {
    println!("Cleaning build artifacts...");
    project.clean()?;
    println!("✓ Project cleaned successfully.");

    Ok(())
}
