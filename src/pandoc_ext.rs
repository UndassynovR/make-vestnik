use std::path::Path;
use std::process::Command;

pub fn run_pandoc<P: AsRef<Path>>(input_path: P) -> String {
    let path = input_path.as_ref();

    // Ensure the input file exists
    if !path.exists() {
        panic!("[PANDOC]: Input file does not exist");
    }

    let input_str = path
        .to_str()
        .expect("[PANDOC]: Input path is not valid UTF-8");

    let output = Command::new("pandoc")
        .arg(input_str)
        .arg("-f")
        .arg("docx")
        .arg("-t")
        .arg("latex")
        .output()
        .expect("[PANDOC]: Failed to execute pandoc command");

    if output.status.success() {
        String::from_utf8(output.stdout).expect("[PANDOC]: Output is not valid UTF-8")
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        panic!("[PANDOC]: Pandoc error:\n{}", err_msg);
    }
}
