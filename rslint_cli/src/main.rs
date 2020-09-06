use codespan_reporting::term::{
    emit,
    termcolor::{ColorChoice, StandardStream},
};
use codespan_reporting::files::Files;
use rayon::prelude::*;
use rslint_cli::{
    codespan_config, lint_err, ExplanationRunner, FileWalker, JsFileKind, panic_hook
};
use rslint_core::{lint_file, CstRuleStore};
use structopt::StructOpt;
use std::panic::set_hook;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rslint",
    about = "An extremely fast and configurable JavaScript linter"
)]
pub(crate) struct Options {
    /// Whether to include potentially spammy details in rule diagnostics.
    #[structopt(short, long)]
    verbose: bool,
    /// A glob pattern to lint.
    #[structopt(default_value = "./")]
    files: String,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
pub(crate) enum SubCommand {
    /// Explain a list of rules, ex: `explain getter-return, no-cond-assign`
    Explain { rules: Vec<String> },
}

fn main() {
    set_hook(Box::new(panic_hook));
    
    let opt = Options::from_args();

    if let Some(SubCommand::Explain { rules }) = opt.cmd {
        ExplanationRunner::new(rules).print();
    } else {
        let res = glob::glob(&opt.files);
        if let Err(err) = res {
            lint_err!("Invalid glob pattern: {}", err);
            return;
        }
        let walker = FileWalker::from_glob(res.unwrap());
        let diagnostics = walker
            .files
            .par_iter()
            .map(|(id, file)| {
                lint_file(
                    *id,
                    &file.source,
                    file.kind == JsFileKind::Module,
                    CstRuleStore::new().builtins(),
                    opt.verbose,
                )
            })
            .flatten()
            .collect::<Vec<_>>();

        for diagnostic in &diagnostics {
            emit(
                &mut StandardStream::stderr(ColorChoice::Always),
                &codespan_config(),
                &walker,
                diagnostic,
            )
            .expect("Failed to throw diagnostic");
        }
    }
}
