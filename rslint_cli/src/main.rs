use rslint_cli::ExplanationRunner;
use structopt::StructOpt;

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
    /// Watch the linted files for changes and lint them on the fly again
    #[structopt(short, long)]
    watch: bool,
}

#[derive(Debug, StructOpt)]
pub(crate) enum SubCommand {
    /// Explain a list of rules, ex: `explain getter-return, no-cond-assign`
    Explain { rules: Vec<String> },
}

fn main() {
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(rslint_cli::panic_hook));

    let opt = Options::from_args();

    if let Some(SubCommand::Explain { rules }) = opt.cmd {
        ExplanationRunner::new(rules).print();
    } else {
        rslint_cli::run(opt.files, opt.verbose, opt.watch);
    }
}
