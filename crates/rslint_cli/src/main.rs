use rslint_cli::ExplanationRunner;
use structopt::{clap::arg_enum, StructOpt};
use yastl::Pool;

const DEV_FLAGS_HELP: &str = "
Developer flags that are used by RSLint developers to debug RSLint.

    -Z help     -- Shows this message
    -Z tokenize -- Tokenizes the input files and dumps the tokens
    -Z dumpast  -- Parses the input files and prints the parsed AST

Run with 'rslint -Z <FLAG> <FILES>'.";

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
    files: Vec<String>,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
    /// Automatically attempt to fix any issues which can be fixed
    #[structopt(short, long)]
    fix: bool,
    /// Attempt to run autofixes even if the code contains syntax errors (may produce weird fixes or more errors)
    #[structopt(short = "D", long)]
    dirty: bool,
    /// Disables the global config that is located in your global config directory.
    #[structopt(long)]
    no_global_config: bool,
    /// Maximum number of threads that will be spawned by RSLint. (default: number of cpu cores)
    #[structopt(long)]
    max_threads: Option<usize>,
    /// The error formatter to use, either "short" or "long" (default)
    #[structopt(short = "F", long)]
    formatter: Option<String>,
    /// Developer only flags. See `-Z help` for more information.
    #[structopt(name = "FLAG", short = "Z")]
    dev_flag: Option<DevFlag>,
}

arg_enum! {
    #[derive(Debug, PartialEq, Eq)]
    enum DevFlag {
        Help,
        Tokenize,
        DumpAst,
    }
}

#[derive(Debug, StructOpt, PartialEq, Eq)]
pub(crate) enum SubCommand {
    /// Explain a list of rules, ex: `explain getter-return, no-cond-assign`
    Explain { rules: Vec<String> },
    /// Show all of the available rules
    // TODO: show only rules of particular groups
    Rules,
    /// Try to infer the options of some rules from various files and print the results
    Infer { files: Vec<String> },
}

fn main() {
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(rslint_cli::panic_hook));

    let opt = Options::from_args();

    let num_threads = opt.max_threads.unwrap_or_else(num_cpus::get);
    let config = yastl::ThreadConfig::new().prefix("rslint-worker");
    let pool = Pool::with_config(num_threads, config);

    execute(opt, pool);
}

fn execute(opt: Options, pool: Pool) {
    match (opt.dev_flag, opt.cmd) {
        (Some(DevFlag::Help), _) => println!("{}", DEV_FLAGS_HELP),
        (Some(DevFlag::Tokenize), _) => rslint_cli::tokenize(opt.files),
        (Some(DevFlag::DumpAst), _) => rslint_cli::dump_ast(opt.files),

        (_, Some(SubCommand::Explain { rules })) => ExplanationRunner::new(rules).print(),
        (_, Some(SubCommand::Rules)) => rslint_cli::show_all_rules(),
        (_, Some(SubCommand::Infer { files })) => rslint_cli::infer(files),
        (_, None) => rslint_cli::run(
            opt.files,
            opt.verbose,
            opt.fix,
            opt.dirty,
            opt.formatter,
            opt.no_global_config,
            pool,
        ),
    }
}
