mod files;

use ascii_table::{AsciiTable, Column};
use colored::Colorize;
use files::*;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rslint_parser::{parse_module, parse_text};
use std::path::PathBuf;

pub fn run(_query: Option<&str>) {
    let files = get_test_files();
    let num_ran = files.len();

    let pb = indicatif::ProgressBar::new(num_ran as u64);
    pb.set_message(&format!("{} tests", "Running".bold().cyan()));
    pb.set_style(default_bar_style());

    std::panic::set_hook(Box::new(|_| {}));
    let start_tests = std::time::Instant::now();
    let res = files
        .into_par_iter()
        .progress_with(pb.clone())
        .map(|file| {
            let res = run_test_file(file);

            if let Some(ref fail) = res.fail {
                let pb = pb.clone();
                let reason = match fail {
                    FailReason::IncorrectlyPassed => "incorrectly passed parsing",
                    FailReason::IncorrectlyErrored => "incorrectly threw an error",
                    FailReason::ParserPanic => "panicked while parsing",
                };
                let msg = format!(
                    "{} '{}' {}",
                    "Test".bold().red(),
                    res.path
                        .strip_prefix("xtask/src/coverage/test262/test/")
                        .unwrap_or(&res.path)
                        .display(),
                    reason.bold()
                );
                pb.println(msg);
            }

            res
        })
        .collect::<Vec<_>>();
    let _ = std::panic::take_hook();

    pb.finish_and_clear();
    println!(
        "\n{} {} tests in {:.2}s\n",
        "Ran".bold().bright_green(),
        num_ran,
        start_tests.elapsed().as_secs_f32()
    );

    let panicked = res
        .iter()
        .filter(|res| res.fail == Some(FailReason::ParserPanic))
        .count();
    let errored = res
        .iter()
        .filter(|res| res.fail == Some(FailReason::IncorrectlyErrored))
        .count();
    let passed = res.iter().filter(|res| res.fail.is_none()).count();

    let mut table = AsciiTable::default();

    let mut counter = 0usize;
    let mut create_column = |name: colored::ColoredString| {
        let mut column = Column::default();
        column.header = name.to_string();
        column.align = ascii_table::Align::Center;
        table.columns.insert(counter, column);
        counter += 1;
    };

    create_column("Tests ran".into());
    create_column("Passed".green());
    create_column("Failed".red());
    create_column("Panics".red());
    create_column("Coverage".cyan());

    let coverage = (passed as f64 / num_ran as f64) * 100.0;
    let coverage = format!("{:.2}", coverage);
    let numbers: Vec<&dyn std::fmt::Display> =
        vec![&num_ran, &passed, &errored, &panicked, &coverage];
    table.print(vec![numbers]);
}

pub fn run_test_file(file: TestFile) -> TestResult {
    let TestFile { code, meta, path } = file;

    if meta.flags.contains(&TestFlag::OnlyStrict) {
        let res = exec_test(code, true, false);
        let fail = passed(res, meta);
        TestResult { fail, path }
    } else if meta.flags.contains(&TestFlag::NoStrict) || meta.flags.contains(&TestFlag::Raw) {
        let res = exec_test(code, false, false);
        let fail = passed(res, meta);
        TestResult { fail, path }
    } else if meta.flags.contains(&TestFlag::Module) {
        let res = exec_test(code, false, true);
        let fail = passed(res, meta);
        TestResult { fail, path }
    } else {
        let l = exec_test(code.clone(), false, false);
        let r = exec_test(code, true, false);
        merge_tests(l, r, meta, path)
    }
}

fn default_bar_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template("{msg} [{bar:40}]")
        .progress_chars("=> ")
}

fn merge_tests(l: ExecRes, r: ExecRes, meta: MetaData, path: PathBuf) -> TestResult {
    let fail = passed(l, meta.clone()).or_else(|| passed(r, meta));
    TestResult { fail, path }
}

fn passed(res: ExecRes, meta: MetaData) -> Option<FailReason> {
    let should_fail = meta
        .negative
        .filter(|neg| neg.phase == Phase::Parse)
        .is_some();

    match res {
        ExecRes::ParserPanic => Some(FailReason::ParserPanic),
        ExecRes::ParseCorrectly if !should_fail => None,
        ExecRes::Errors if should_fail => None,
        ExecRes::ParseCorrectly if should_fail => Some(FailReason::IncorrectlyPassed),
        ExecRes::Errors if !should_fail => Some(FailReason::IncorrectlyErrored),
        _ => unreachable!(),
    }
}

enum ExecRes {
    Errors,
    ParseCorrectly,
    ParserPanic,
}

fn exec_test(mut code: String, strict: bool, module: bool) -> ExecRes {
    if strict {
        code.insert_str(0, "\"use strict\";\n");
    }

    let result = std::panic::catch_unwind(|| {
        if module {
            parse_module(&code, 0).ok().is_ok()
        } else {
            parse_text(&code, 0).ok().is_ok()
        }
    });

    let result = result
        .map(|res| {
            if res {
                ExecRes::ParseCorrectly
            } else {
                ExecRes::Errors
            }
        })
        .unwrap_or(ExecRes::ParserPanic);

    result
}
