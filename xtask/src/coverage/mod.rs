mod files;
mod tablegen;

use files::*;
use rslint_parser::{parse_module, parse_text};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread::Builder;
use std::time::Duration;
use tablegen::{Cell, Table};
use termcolor::Color;

pub fn run() {
    let files = get_test_files().collect::<Vec<_>>();
    let num_ran = files.len();

    let res = files.into_iter().map(run_test_file).collect::<Vec<_>>();

    let infinitely_recursed = res
        .iter()
        .filter(|res| res.fail == Some(FailReason::InfiniteRecursion))
        .count();
    let errored = res
        .iter()
        .filter(|res| res.fail == Some(FailReason::IncorrectlyErrored))
        .count();
    let passed = res.iter().filter(|res| res.fail.is_none()).count();

    let header = vec![
        Cell::new("Tests ran"),
        Cell::with_color("Passed", Color::Green),
        Cell::with_color("Failed", Color::Red),
        Cell::with_color("Panics", Color::Red),
        Cell::with_color("Coverage", Color::Cyan),
    ];

    let numbers = vec![num_ran, passed, errored, infinitely_recursed]
        .into_iter()
        .map(|x| Cell::new(x.to_string()))
        .collect::<Vec<_>>();

    let coverage = (passed as f64 / num_ran as f64) * 100.0;
    Table::new(
        header,
        vec![[
            numbers,
            vec![Cell::new(format!("{}%", coverage.round().to_string()))],
        ]
        .concat()],
        vec![],
    )
    .render();

    let rows = res
        .iter()
        .filter(|res| res.fail == Some(FailReason::IncorrectlyErrored))
        .map(|res| {
            format!(
                "{:#?}: should fail: {}",
                res.path,
                res.fail == Some(FailReason::IncorrectlyPassed)
            )
        });

    println!("{}", rows.collect::<Vec<_>>().join("\n"));
}

pub fn run_test_file(file: TestFile) -> TestResult {
    let TestFile { code, meta, path } = file;

    if meta.flags.contains(&TestFlag::OnlyStrict) {
        let res = exec_test(code, true, false, path.clone());
        let fail = passed(res, meta);
        TestResult { fail, path }
    } else if meta.flags.contains(&TestFlag::NoStrict) || meta.flags.contains(&TestFlag::Raw) {
        let res = exec_test(code, false, false, path.clone());
        let fail = passed(res, meta);
        TestResult { fail, path }
    } else if meta.flags.contains(&TestFlag::Module) {
        let res = exec_test(code, false, true, path.clone());
        let fail = passed(res, meta);
        TestResult { fail, path }
    } else {
        let l = exec_test(code.clone(), false, false, path.clone());
        let r = exec_test(code, true, false, path.clone());
        merge_tests(l, r, meta, path)
    }
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
        ExecRes::InfiniteRecursion => Some(FailReason::InfiniteRecursion),
        ExecRes::ParseCorrectly if !should_fail => None,
        ExecRes::Errors if should_fail => None,
        ExecRes::ParseCorrectly if should_fail => Some(FailReason::IncorrectlyPassed),
        ExecRes::Errors if !should_fail => Some(FailReason::IncorrectlyErrored),
        _ => unreachable!(),
    }
}

enum ExecRes {
    Errors,
    InfiniteRecursion,
    ParseCorrectly,
}

fn exec_test(mut code: String, strict: bool, module: bool, path: PathBuf) -> ExecRes {
    if strict {
        code.insert_str(0, "\"use strict\";\n");
    }

    let (sender, receiver) = channel();
    let _ = Builder::new().name(format!("{:#?}", path)).spawn(move || {
        let parse_func: Box<dyn Fn() -> bool> = if module {
            Box::new(|| {
                let parse = parse_module(&code, 0);
                parse.ok().is_ok()
            })
        } else {
            Box::new(|| {
                let parse = parse_text(&code, 0);
                parse.ok().is_ok()
            })
        };

        sender
            .send(parse_func())
            .expect("Failed to send to receiver");
    });

    receiver
        .recv_timeout(Duration::from_millis(500))
        .map(|res| {
            if res {
                ExecRes::ParseCorrectly
            } else {
                ExecRes::Errors
            }
        })
        .unwrap_or(ExecRes::InfiniteRecursion)
}
