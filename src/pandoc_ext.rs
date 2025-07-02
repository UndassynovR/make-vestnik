use pandoc::{InputFormat, OutputFormat, OutputKind, PandocOutput};
use std::path::Path;

pub fn run_pandoc<P: AsRef<Path>>(input_path: P) -> String {
    let path = input_path.as_ref();
    let input_str = path
        .to_str()
        .expect("[PANDOC]: Input path is not valid UTF-8");

    let mut pandoc = pandoc::new();
    pandoc.set_input_format(InputFormat::Docx, vec![]);
    pandoc.set_output_format(OutputFormat::Latex, vec![]);
    pandoc.add_input(input_str);
    pandoc.set_output(OutputKind::Pipe);

    match pandoc.execute() {
        Ok(PandocOutput::ToBuffer(s)) => s,
        _ => panic!(
            "[PANDOC]: Expected LaTeX output in memory buffer (ToBuffer), but got something else"
        ),
    }
}
