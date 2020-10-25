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
    /// Automatically attempt to fix any issues which can be fixed
    #[structopt(short, long)]
    fix: bool,
    /// Attempt to run autofixes even if the code contains syntax errors (may produce weird fixes or more errors)
    #[structopt(short = "D", long)]
    dirty: bool,
    /// The error formatter to use, either "short" or "long" (default)
    #[structopt(short = "F", long)]
    formatter: Option<String>,
}

#[derive(Debug, StructOpt, PartialEq, Eq)]
pub(crate) enum SubCommand {
    /// Explain a list of rules, ex: `explain getter-return, no-cond-assign`
    Explain { rules: Vec<String> },
    /// Show all of the available rules
    // TODO: show only rules of particular groups
    Rules,
}

fn main() {
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(rslint_cli::panic_hook));

    let opt = Options::from_args();

    if let Some(SubCommand::Explain { rules }) = opt.cmd {
        ExplanationRunner::new(rules).print();
    } else if opt.cmd == Some(SubCommand::Rules) {
        rslint_cli::show_all_rules();
    } else {
        rslint_cli::run(opt.files, opt.verbose, opt.fix, opt.dirty, opt.formatter);
    }
}
