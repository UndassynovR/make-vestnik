mod images;
mod latex_ext;
mod pandoc_ext;
mod project;
mod util;
use project::*;

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Show brief usage for no arguments
    if args.len() < 2 {
        println!("Usage: make-vestnik <action> [options]...\nTry 'make-vestnik --help' for more information.");
        return Ok(());
    }
    
    // Show detailed help for help flag
    if args.get(1).map_or(false, |arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }
    
    let action = args[1].as_str();
    
    match action {
        "create" | "update" => handle_create_update(&args)?,
        "compile" => handle_compile(&args)?,
        _ => {
            eprintln!("Error: Unknown action '{}'. Available actions: create, update, compile. Use --help for more information.", action);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!("Document Project Manager - make-vestnik

USAGE:
    make-vestnik <action> [options]

ACTIONS:
    create <project_dir> <docx_file>    Create new project from DOCX
    update <project_dir> <docx_file>    Update existing project with DOCX
    compile [project_dir]               Compile project (watch mode)

EXAMPLES:
    make-vestnik create ./my-project document.docx
    make-vestnik update ./my-project updated.docx
    make-vestnik compile ./my-project
    make-vestnik compile                          # Uses current directory

OPTIONS:
    -h, --help                          Show this help message");
}

fn handle_create_update(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.len() < 4 {
        eprintln!("Error: {} action requires both project directory and DOCX file. Usage: make-vestnik {} <project_dir> <docx_file>", 
            args[1], args[1]);
        std::process::exit(1);
    }
    
    let action = &args[1];
    
    // Parse arguments to identify docx file and project directory (order doesn't matter)
    let mut docx_file = None;
    let mut project_dir = None;
    
    for arg in &args[2..] {
        if arg.ends_with(".docx") {
            docx_file = Some(arg);
        } else {
            project_dir = Some(arg);
        }
    }
    
    let docx_file = docx_file.ok_or("Error: Missing .docx file")?;
    let project_dir = project_dir.ok_or("Error: Missing project directory")?;
    
    // Check if DOCX file exists
    if !PathBuf::from(docx_file).exists() {
        eprintln!("Error: DOCX file '{}' not found.", docx_file);
        std::process::exit(1);
    }
    
    println!("Starting {} operation with project directory '{}' and DOCX file '{}'", action, project_dir, docx_file);
    
    if action == "create" {
        println!("Creating new project...");
        create_project(project_dir)?;
        println!("Project created successfully.");
    }
    
    println!("Updating project with DOCX content...");
    update_project(docx_file, project_dir)?;
    println!("Project updated successfully.");
    
    Ok(())
}

fn handle_compile(args: &[String]) -> Result<(), Box<dyn Error>> {
    // Check for .docx files in compile arguments (common mistake)
    for arg in &args[2..] {
        if arg.ends_with(".docx") {
            eprintln!("Error: Compile action cannot process DOCX files. Did you mean to use 'create' or 'update' instead?");
            std::process::exit(1);
        }
    }
    
    let project_dir = if args.len() > 2 {
        let dir = PathBuf::from(&args[2]);
        if !dir.exists() {
            eprintln!("Error: Directory '{}' not found.", args[2]);
            std::process::exit(1);
        }
        Some(dir)
    } else {
        let current = env::current_dir().map_err(|e| {
            format!("Failed to get current directory: {}", e)
        })?;
        println!("Using current directory: {}", current.display());
        Some(current)
    };
    
    let dir_msg = match &project_dir {
        Some(dir) => format!("Starting compilation in watch mode for directory '{}'. Press Ctrl+C to stop.", dir.display()),
        None => "Starting compilation in watch mode. Press Ctrl+C to stop.".to_string(),
    };
    println!("{}", dir_msg);
    
    watch_and_compile_project(project_dir)?;
    
    Ok(())
}

