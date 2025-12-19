use anyhow::Result;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

pub fn extract_images_from_docx<P: AsRef<Path>, Q: AsRef<Path>>(
    docx_path: P,
    output_dir: Q,
) -> Result<()> {
    let docx_path = docx_path.as_ref();
    let output_dir = output_dir.as_ref();

    let file = File::open(docx_path)?;
    let mut archive = ZipArchive::new(file).map_err(|e| anyhow::anyhow!("ZIP error: {}", e))?;

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Iterate over files inside the .docx (which is a zip)
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Extract only image files inside word/media/
        if name.starts_with("word/media/") {
            let filename = Path::new(&name)
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

            let output_path = output_dir.join(filename);
            let mut out_file = File::create(&output_path)?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            out_file.write_all(&buffer)?;
        }
    }

    // Collect paths first to avoid modifying directory while iterating
    let mut files_to_convert: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();

            // Convert WMF, EMF, GIF, and TIFF to PNG
            if matches!(ext_lower.as_str(), "wmf" | "emf" | "gif" | "tiff" | "tif") {
                files_to_convert.push(path);
            }
        }
    }

    // Now convert collected files
    for path in files_to_convert {
        if let Err(e) = convert_to_png(&path) {
            eprintln!("⚠ Warning: Failed to convert {}: {}", path.display(), e);
        }
    }

    Ok(())
}

fn convert_to_png(path: &Path) -> Result<()> {
    let png_path = path.with_extension("png");

    let status = Command::new("magick")
        .args(&[
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
            "-density",
            "300",
            "-background",
            "white",
            "-alpha",
            "remove",
            "-flatten",
            png_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
        ])
        .status()?;

    if status.success() {
        // Remove original file after successful conversion
        fs::remove_file(path)?;
        println!(
            "✓ Converted {} to PNG",
            path.file_name().unwrap().to_str().unwrap()
        );
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "ImageMagick conversion failed for {}",
            path.display()
        ))
    }
}
