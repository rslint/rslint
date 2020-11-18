#![deny(rust_2018_idioms)]

mod cli;
mod files;
mod macros;
mod panic_hook;

pub use self::{
    cli::{show_all_rules, ExplanationRunner},
    files::*,
    panic_hook::panic_hook,
};

use colored::Colorize;
use rslint_core::{
    errors::{file, Diagnostic, Emitter},
    lexer::Lexer,
};

pub async fn dump_ast(files: Vec<String>) {
    FileWalker::walk_files(files.as_ref(), |file| async move {
        println!("{}", file.path_or_name().red().bold());
        println!("{:#?}", file.parse());
    })
    .await
}

pub async fn tokenize(files: Vec<String>) {
    FileWalker::walk_files(files.as_ref(), |file| async move {
        println!("{}", file.path_or_name().red().bold());

        let source = file.source.as_str();
        let tokens = Lexer::from_str(source, file.id)
            .map(|(tok, d)| {
                if let Some(d) = d {
                    emit_diagnostic(&d, &file);
                }
                tok
            })
            .collect::<Vec<_>>();

        rslint_core::parser::TokenSource::new(source, tokens.as_slice()).for_each(|tok| {
            println!("{:?}@{}..{}", tok.kind, tok.range.start, tok.range.end);
        });
        println!();
    })
    .await
}

pub fn emit_diagnostic(diagnostic: &Diagnostic, walker: &dyn file::Files) {
    let mut emitter = Emitter::new(walker);
    emitter
        .emit_stderr(&diagnostic, true)
        .expect("failed to throw linter diagnostic")
}
