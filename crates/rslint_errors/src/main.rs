use rslint_errors::{
    file::{FileId, Files},
    Applicability, Diagnostic, Severity,
};

struct SingleFile {
    name: String,
    source: String,
}

impl Files for SingleFile {
    fn name(&self, _id: FileId) -> &str {
        &self.name
    }

    fn source(&self, _id: FileId) -> &str {
        &self.source
    }

    fn line_index(&self, byte_index: usize) -> usize {
        let starts = rslint_errors::file::line_starts(&self.source).collect::<Vec<_>>();
        match starts.binary_search(&byte_index) {
            Ok(line) => line,
            Err(next_line) => next_line - 1,
        }
    }
}

fn main() {
    let file = SingleFile {
        name: "src/main.rs".to_string(),
        source: "console.log();\nconsole.log();;".to_string(),
    };

    let len = file.source.len();
    let d = Diagnostic::new(0, Severity::Error, "unexpected `;`")
        .primary(len - 1..len, "".to_string())
        .suggestion(
            len - 1..len,
            "consider removing this semicolon",
            "".to_string(),
            Applicability::Always,
        );
    let out = std::io::stdout();
    let mut lock = out.lock();
    //rslint_errors::emit(&mut lock, &file, true, vec![d]).unwrap();
}
