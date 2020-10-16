use rslint_errors::{
    file::{FileId, Files},
    Applicability, Diagnostic, Emitter, Severity,
};

struct SingleFile {
    name: String,
    source: String,
}

impl Files for SingleFile {
    fn name(&self, _id: FileId) -> Option<&str> {
        Some(&self.name)
    }

    fn source(&self, _id: FileId) -> &str {
        &self.source
    }

    fn line_index(&self, _file: FileId, byte_index: usize) -> usize {
        let starts = rslint_errors::file::line_starts(&self.source).collect::<Vec<_>>();
        starts
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1)
    }
}

fn main() {
    let file = SingleFile {
        name: "src/main.rs".to_string(),
        source: "if foo {}\nsome".to_string(),
    };

    let d = Diagnostic::new(0, Severity::Error, "condition not wrapped in parenthesis")
        .primary(3usize..6, "".to_string())
        .suggestion(
            3usize..6,
            "consider wrapping this in parenthesis",
            "(foo)".to_string(),
            Applicability::Always,
        );

    let mut emitter = Emitter::stdout(Box::new(file), true);
    emitter.emit_diagnostic(&d).unwrap();
}
