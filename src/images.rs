use anyhow::Result;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;

pub fn extract_images_from_docx<P: AsRef<Path>, Q: AsRef<Path>>(
    docx_path: P,
    output_dir: Q,
) -> Result<()> {
    let docx_path = docx_path.as_ref();
    let output_dir = output_dir.as_ref();

    let file = File::open(docx_path)?;
    let mut archive = ZipArchive::new(file)?;

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

    Ok(())
}
