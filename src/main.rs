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

    if args.len() < 2 {
        return Err("Usage: <program> <action> <path1> [path2]".into());
    }

    let action = args[1].as_str();

    let mut docx_file = None;
    let mut dirpath = None;

    for arg in &args[2..] {
        if arg.ends_with(".docx") {
            docx_file = Some(arg);
        } else {
            dirpath = Some(arg);
        }
    }

    match action {
        "create" | "update" => {
            let docx = docx_file.ok_or("Missing .docx file")?;
            let dir = dirpath.ok_or("Missing project directory")?;

            if action == "create" {
                create_project(dir)?;
            }
            update_project(docx, dir)?;
        }
        "compile" => {
            // Reject .docx files passed to compile
            if docx_file.is_some() {
                return Err("Compile action does not accept .docx files".into());
            }

            // Use provided directory if exists, else current directory
            let dir = dirpath
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

            watch_and_compile_project(Some(dir))?;
        }
        _ => return Err(format!("Unknown action '{}'", action).into()),
    }

    Ok(())
}
