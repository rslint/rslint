use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use differential_datalog::{ddval::DDValue, program::Update};
use rslint_scope::{
    globals::{BUILTIN, ES2021, NODE},
    FileId, ScopeAnalyzer,
};
use std::{ffi::OsStr, fs};
use walkdir::WalkDir;

const EXPRESSION_PARSER: &str = include_str!("source/ExpressionParser.mjs");
const STATEMENT_PARSER: &str = include_str!("source/StatementParser.mjs");
const REGEX_EXPR: &str = include_str!("source/RegExp.mjs");
const ENVIRONMENT: &str = include_str!("source/environment.mjs");
const REGEX_EXPR_PARSER: &str = include_str!("source/RegExpParser.mjs");
const TYPED_ARRAY_CONSTRUCTOR: &str = include_str!("source/TypedArrayConstructor.mjs");
const ARRAY_PROTOTYPE: &str = include_str!("source/ArrayPrototype.mjs");
const ARRAY_PROTOTYPE_SHARED: &str = include_str!("source/ArrayPrototypeShared.mjs");
const PROXY_OBJECTS: &str = include_str!("source/proxy_objects.mjs");
const UNICODE: &str = include_str!("source/Unicode.mjs");

#[track_caller]
fn enabled_analyzer(workers: usize) -> ScopeAnalyzer {
    let analyzer = ScopeAnalyzer::new(black_box(workers)).expect("failed to create scope analyzer");
    for id in (0..300).map(FileId::new) {
        analyzer.no_shadow(id, Some(Default::default()));
        analyzer.no_typeof_undef(id, Some(Default::default()));
        analyzer.no_undef(id, Some(Default::default()));
        analyzer.no_unused_labels(id, Some(Default::default()));
        analyzer.no_unused_vars(id, Some(Default::default()));
        analyzer.no_use_before_def(id, Some(Default::default()));
    }

    analyzer.flush_config_queue().unwrap();
    analyzer
}

fn initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("create a new analyzer");
    for i in 1..=6 {
        let param = format!("{} thread{}", i, if i == 1 { "" } else { "s" });

        group.bench_with_input(
            BenchmarkId::new("analyzer initialization", param),
            &(),
            |b, _| b.iter(|| enabled_analyzer(i)),
        );
    }
}

fn single_file_processing(c: &mut Criterion) {
    let syntax_node = rslint_parser::parse_text(EXPRESSION_PARSER, 0).syntax();

    let mut group = c.benchmark_group("process single file, empty instance");
    for i in 1..=6 {
        let param = format!("{} thread{}", i, if i == 1 { "" } else { "s" });

        group.bench_with_input(
            BenchmarkId::new("single file", param),
            &syntax_node,
            |b, syntax_node| {
                b.iter_batched(
                    || enabled_analyzer(i),
                    |analyzer| {
                        analyzer
                            .analyze(black_box(FileId::new(0)), black_box(syntax_node))
                            .unwrap();

                        analyzer
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
}

fn multiple_file_processing(c: &mut Criterion) {
    let syntax_nodes = &[
        rslint_parser::parse_text(EXPRESSION_PARSER, 0).syntax(),
        rslint_parser::parse_text(STATEMENT_PARSER, 1).syntax(),
        rslint_parser::parse_text(REGEX_EXPR, 2).syntax(),
        rslint_parser::parse_text(ENVIRONMENT, 3).syntax(),
        rslint_parser::parse_text(REGEX_EXPR_PARSER, 4).syntax(),
        rslint_parser::parse_text(TYPED_ARRAY_CONSTRUCTOR, 5).syntax(),
    ];

    let mut group = c.benchmark_group("process multiple files, empty instance");
    for i in 1..=6 {
        let param = format!("{} thread{}", i, if i == 1 { "" } else { "s" });

        group.bench_with_input(
            BenchmarkId::new("multiple files", param),
            syntax_nodes,
            |b, syntax_nodes| {
                b.iter_batched(
                    || enabled_analyzer(i),
                    |analyzer| {
                        for (i, syntax_node) in syntax_nodes.iter().enumerate() {
                            analyzer
                                .analyze(black_box(FileId::new(i as u32)), black_box(syntax_node))
                                .unwrap();
                        }

                        analyzer
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
}

fn progressive_reprocessing_n5(c: &mut Criterion) {
    let syntax_node1 = rslint_parser::parse_text(EXPRESSION_PARSER, 0).syntax();
    let syntax_node2 = rslint_parser::parse_text(STATEMENT_PARSER, 1).syntax();
    let syntax_node3 = rslint_parser::parse_text(REGEX_EXPR, 2).syntax();
    let syntax_node4 = rslint_parser::parse_text(ENVIRONMENT, 3).syntax();
    let syntax_node5 = rslint_parser::parse_text(REGEX_EXPR_PARSER, 4).syntax();
    let syntax_node6 = rslint_parser::parse_text(TYPED_ARRAY_CONSTRUCTOR, 5).syntax();

    let routine = |analyzer: ScopeAnalyzer| {
        analyzer
            .analyze(FileId::new(0), black_box(&syntax_node1))
            .unwrap();
        analyzer
    };

    let process_files = |analyzer: &ScopeAnalyzer| {
        analyzer
            .analyze(FileId::new(1), black_box(&syntax_node2))
            .unwrap();
        analyzer
            .analyze(FileId::new(2), black_box(&syntax_node3))
            .unwrap();
        analyzer
            .analyze(FileId::new(3), black_box(&syntax_node4))
            .unwrap();
        analyzer
            .analyze(FileId::new(4), black_box(&syntax_node5))
            .unwrap();
        analyzer
            .analyze(FileId::new(5), black_box(&syntax_node6))
            .unwrap();
    };

    c.benchmark_group("process the 5th file")
        .bench_function("1 thread", |b| {
            b.iter_batched(
                || {
                    let analyzer = enabled_analyzer(1);
                    process_files(&analyzer);

                    analyzer
                },
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("2 threads", |b| {
            b.iter_batched(
                || {
                    let analyzer = enabled_analyzer(2);
                    process_files(&analyzer);

                    analyzer
                },
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("3 threads", |b| {
            b.iter_batched(
                || {
                    let analyzer = enabled_analyzer(3);
                    process_files(&analyzer);

                    analyzer
                },
                routine,
                BatchSize::PerIteration,
            )
        });
}

fn progressive_reprocessing_n10(c: &mut Criterion) {
    let syntax_node1 = rslint_parser::parse_text(EXPRESSION_PARSER, 0).syntax();
    let syntax_node2 = rslint_parser::parse_text(STATEMENT_PARSER, 1).syntax();
    let syntax_node3 = rslint_parser::parse_text(REGEX_EXPR, 2).syntax();
    let syntax_node4 = rslint_parser::parse_text(ENVIRONMENT, 3).syntax();
    let syntax_node5 = rslint_parser::parse_text(REGEX_EXPR_PARSER, 4).syntax();
    let syntax_node6 = rslint_parser::parse_text(TYPED_ARRAY_CONSTRUCTOR, 5).syntax();
    let syntax_node7 = rslint_parser::parse_text(ARRAY_PROTOTYPE, 6).syntax();
    let syntax_node8 = rslint_parser::parse_text(ARRAY_PROTOTYPE_SHARED, 7).syntax();
    let syntax_node9 = rslint_parser::parse_text(PROXY_OBJECTS, 8).syntax();
    let syntax_node10 = rslint_parser::parse_text(UNICODE, 9).syntax();

    let routine = |analyzer: ScopeAnalyzer| {
        analyzer
            .analyze(FileId::new(0), black_box(&syntax_node1))
            .unwrap();
        analyzer
    };

    let process_files = |analyzer: &ScopeAnalyzer| {
        analyzer
            .analyze(FileId::new(1), black_box(&syntax_node2))
            .unwrap();
        analyzer
            .analyze(FileId::new(2), black_box(&syntax_node3))
            .unwrap();
        analyzer
            .analyze(FileId::new(3), black_box(&syntax_node4))
            .unwrap();
        analyzer
            .analyze(FileId::new(4), black_box(&syntax_node5))
            .unwrap();
        analyzer
            .analyze(FileId::new(5), black_box(&syntax_node6))
            .unwrap();
        analyzer
            .analyze(FileId::new(6), black_box(&syntax_node7))
            .unwrap();
        analyzer
            .analyze(FileId::new(7), black_box(&syntax_node8))
            .unwrap();
        analyzer
            .analyze(FileId::new(8), black_box(&syntax_node9))
            .unwrap();
        analyzer
            .analyze(FileId::new(9), black_box(&syntax_node10))
            .unwrap();
    };

    c.benchmark_group("process the 10th file")
        .bench_function("1 thread", |b| {
            b.iter_batched(
                || {
                    let analyzer = enabled_analyzer(1);
                    process_files(&analyzer);

                    analyzer
                },
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("2 threads", |b| {
            b.iter_batched(
                || {
                    let analyzer = enabled_analyzer(2);
                    process_files(&analyzer);

                    analyzer
                },
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("3 threads", |b| {
            b.iter_batched(
                || {
                    let analyzer = enabled_analyzer(3);
                    process_files(&analyzer);

                    analyzer
                },
                routine,
                BatchSize::PerIteration,
            )
        });
}

fn process_multiple_files_batched(c: &mut Criterion) {
    let syntax_node1 = rslint_parser::parse_text(EXPRESSION_PARSER, 0).syntax();
    let syntax_node2 = rslint_parser::parse_text(STATEMENT_PARSER, 1).syntax();
    let syntax_node3 = rslint_parser::parse_text(REGEX_EXPR, 2).syntax();
    let syntax_node4 = rslint_parser::parse_text(ENVIRONMENT, 3).syntax();
    let syntax_node5 = rslint_parser::parse_text(REGEX_EXPR_PARSER, 4).syntax();
    let syntax_node6 = rslint_parser::parse_text(TYPED_ARRAY_CONSTRUCTOR, 5).syntax();
    let syntax_node7 = rslint_parser::parse_text(ARRAY_PROTOTYPE, 6).syntax();
    let syntax_node8 = rslint_parser::parse_text(ARRAY_PROTOTYPE_SHARED, 7).syntax();
    let syntax_node9 = rslint_parser::parse_text(PROXY_OBJECTS, 8).syntax();
    let syntax_node10 = rslint_parser::parse_text(UNICODE, 9).syntax();

    let nodes = (
        syntax_node1,
        syntax_node2,
        syntax_node3,
        syntax_node4,
        syntax_node5,
        syntax_node6,
        syntax_node7,
        syntax_node8,
        syntax_node9,
        syntax_node10,
    );

    let routine = |(
        analyzer,
        (
            syntax_node1,
            syntax_node2,
            syntax_node3,
            syntax_node4,
            syntax_node5,
            syntax_node6,
            syntax_node7,
            syntax_node8,
            syntax_node9,
            syntax_node10,
        ),
    ): (ScopeAnalyzer, _)| {
        analyzer
            .analyze_batch(black_box(&[
                (FileId::new(0), black_box(syntax_node1)),
                (FileId::new(1), black_box(syntax_node2)),
                (FileId::new(2), black_box(syntax_node3)),
                (FileId::new(3), black_box(syntax_node4)),
                (FileId::new(4), black_box(syntax_node5)),
                (FileId::new(5), black_box(syntax_node6)),
                (FileId::new(6), black_box(syntax_node7)),
                (FileId::new(7), black_box(syntax_node8)),
                (FileId::new(8), black_box(syntax_node9)),
                (FileId::new(9), black_box(syntax_node10)),
            ]))
            .unwrap();
        analyzer
    };

    c.benchmark_group("process 10 files batched")
        .sample_size(10)
        .bench_function("1 thread", |b| {
            b.iter_batched(
                || (enabled_analyzer(1), nodes.clone()),
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("2 threads", |b| {
            b.iter_batched(
                || (enabled_analyzer(2), nodes.clone()),
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("3 threads", |b| {
            b.iter_batched(
                || (enabled_analyzer(3), nodes.clone()),
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("4 threads", |b| {
            b.iter_batched(
                || (enabled_analyzer(4), nodes.clone()),
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("5 threads", |b| {
            b.iter_batched(
                || (enabled_analyzer(5), nodes.clone()),
                routine,
                BatchSize::PerIteration,
            )
        })
        .bench_function("6 threads", |b| {
            b.iter_batched(
                || (enabled_analyzer(6), nodes.clone()),
                routine,
                BatchSize::PerIteration,
            )
        });
}

fn collect_engine262_files() -> Vec<Vec<Update<DDValue>>> {
    let mut files: Vec<_> = WalkDir::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/source/engine262/src",
    ))
    .into_iter()
    .filter_entry(|e| e.file_type().is_file() || e.file_type().is_dir())
    .filter_map(Result::ok)
    .filter_map(|file| {
        if file.file_type().is_file()
            && (file.path().extension() == Some(OsStr::new("js"))
                || file.path().extension() == Some(OsStr::new("mjs")))
        {
            Some(fs::read_to_string(file.path()).unwrap())
        } else {
            None
        }
    })
    .collect();
    files.sort_unstable_by_key(|file| file.len());

    files
        .into_iter()
        .enumerate()
        .map(|(id, file)| {
            rslint_scope::make_updates(
                FileId::new(id as u32),
                &rslint_parser::parse_text(&file, id).syntax(),
            )
        })
        .collect()
}

fn coalescing_engine262_files(c: &mut Criterion) {
    let mut files = collect_engine262_files();
    let first = files.pop().unwrap();

    let mut group = c.benchmark_group("engine262");
    group
        .throughput(Throughput::Elements(first.len() as u64))
        .sample_size(10);

    for num_threads in 1..4 {
        for i in 0..files.len() {
            group.bench_with_input(
                BenchmarkId::new(
                    "coalescing analysis times",
                    &format!(
                        "{} thread{}, {} pre-processed files",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                        i,
                    ),
                ),
                &first,
                |b, file| {
                    b.iter_batched(
                        || {
                            let analyzer = enabled_analyzer(num_threads);
                            analyzer.analyze_raw_batch(files[..i].to_owned()).unwrap();

                            (analyzer, file.to_owned())
                        },
                        |(analyzer, file)| analyzer.analyze_raw(black_box(file)).unwrap(),
                        BatchSize::PerIteration,
                    )
                },
            );
        }
    }
}

fn batched_engine262_files(c: &mut Criterion) {
    let files = collect_engine262_files();

    let mut group = c.benchmark_group("engine262");
    group.sample_size(10);

    for num_threads in 1..4 {
        for i in 0..files.len() {
            group.bench_with_input(
                BenchmarkId::new(
                    "batched analysis times",
                    &format!(
                        "{} thread{}, {} files",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                        i,
                    ),
                ),
                &files[..i],
                |b, _| {
                    b.iter_batched(
                        || (enabled_analyzer(num_threads), files.to_vec()),
                        |(analyzer, batch)| analyzer.analyze_raw_batch(black_box(batch)).unwrap(),
                        BatchSize::PerIteration,
                    )
                },
            );
        }
    }
}

fn batched_engine262_files_throughput(c: &mut Criterion) {
    let files = collect_engine262_files();

    for num_threads in 1..4 {
        for i in 0..files.len() {
            let mut group = c.benchmark_group("engine262");
            group.sample_size(10);

            group.throughput(Throughput::Elements(
                files[..i].iter().map(|updates| updates.len() as u64).sum(),
            ));

            group.bench_with_input(
                BenchmarkId::new(
                    "batched analysis throughput",
                    format!(
                        "{} thread{}, {} files",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                        i,
                    ),
                ),
                &files[..i],
                |b, batch| {
                    b.iter_batched(
                        || (enabled_analyzer(num_threads), batch.to_vec()),
                        |(analyzer, batch)| analyzer.analyze_raw_batch(black_box(batch)).unwrap(),
                        BatchSize::PerIteration,
                    )
                },
            );
        }
    }
}

fn engine262_all_lints_ascending(c: &mut Criterion) {
    let files = collect_engine262_files();

    for num_threads in 1..=6 {
        let mut group = c.benchmark_group("lints");
        group.sample_size(10);

        group.bench_with_input(
            BenchmarkId::new(
                format!("all lints ({} files from least to greatest)", files.len()),
                format!(
                    "{} thread{}",
                    num_threads,
                    if num_threads == 1 { "" } else { "s" },
                ),
            ),
            &files,
            |b, batch| {
                b.iter_batched(
                    || (enabled_analyzer(num_threads), batch.to_owned()),
                    |(analyzer, batch)| {
                        black_box(analyzer)
                            .analyze_raw_batch(black_box(batch))
                            .unwrap()
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
}

fn engine262_all_lints_descending(c: &mut Criterion) {
    let files = collect_engine262_files();

    for num_threads in 1..=6 {
        let mut group = c.benchmark_group("lints");
        group.sample_size(10);

        group.bench_with_input(
            BenchmarkId::new(
                format!("all lints ({} files from greatest to least)", files.len()),
                format!(
                    "{} thread{}",
                    num_threads,
                    if num_threads == 1 { "" } else { "s" },
                ),
            ),
            &files,
            |b, batch| {
                b.iter_batched(
                    || (enabled_analyzer(num_threads), batch.to_owned()),
                    |(analyzer, batch)| {
                        black_box(analyzer)
                            .analyze_raw_batch(black_box(batch))
                            .unwrap()
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
}

fn engine262_all_lints_ascending_with_globals(c: &mut Criterion) {
    let files = collect_engine262_files();

    for num_threads in 1..=6 {
        let mut group = c.benchmark_group("lints");
        group.sample_size(10);

        group.bench_with_input(
            BenchmarkId::new(
                format!(
                    "all lints with globals ({} files from least to greatest)",
                    files.len(),
                ),
                format!(
                    "{} thread{}",
                    num_threads,
                    if num_threads == 1 { "" } else { "s" },
                ),
            ),
            &files,
            |b, batch| {
                b.iter_batched(
                    || {
                        (
                            {
                                let analyzer = enabled_analyzer(num_threads);
                                analyzer.inject_globals(BUILTIN).unwrap();
                                analyzer.inject_globals(ES2021).unwrap();
                                analyzer.inject_globals(NODE).unwrap();

                                analyzer
                            },
                            batch.to_owned(),
                        )
                    },
                    |(analyzer, batch)| {
                        black_box(analyzer)
                            .analyze_raw_batch(black_box(batch))
                            .unwrap()
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
}

fn engine262_all_lints_descending_with_globals(c: &mut Criterion) {
    let files = collect_engine262_files();

    for num_threads in 1..=6 {
        let mut group = c.benchmark_group("lints");
        group.sample_size(10);

        group.bench_with_input(
            BenchmarkId::new(
                format!(
                    "all lints with globals ({} files from greatest to least)",
                    files.len(),
                ),
                format!(
                    "{} thread{}",
                    num_threads,
                    if num_threads == 1 { "" } else { "s" },
                ),
            ),
            &files,
            |b, batch| {
                b.iter_batched(
                    || {
                        (
                            {
                                let analyzer = enabled_analyzer(num_threads);
                                analyzer.inject_globals(BUILTIN).unwrap();
                                analyzer.inject_globals(ES2021).unwrap();
                                analyzer.inject_globals(NODE).unwrap();

                                analyzer
                            },
                            batch.to_owned(),
                        )
                    },
                    |(analyzer, batch)| {
                        black_box(analyzer)
                            .analyze_raw_batch(black_box(batch))
                            .unwrap()
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
}

fn engine262_single_lints(c: &mut Criterion) {
    let files = collect_engine262_files();

    for num_threads in 1..=6 {
        let mut group = c.benchmark_group("lints");
        group.sample_size(10);

        group
            .bench_with_input(
                BenchmarkId::new(
                    format!("no shadow ({} files)", files.len()),
                    format!(
                        "{} thread{}",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                    ),
                ),
                &files,
                |b, batch| {
                    b.iter_batched(
                        || {
                            (
                                {
                                    let analyzer = ScopeAnalyzer::new(num_threads).unwrap();
                                    for id in (0..300).map(FileId::new) {
                                        analyzer.no_shadow(id, Some(Default::default()));
                                    }

                                    analyzer.flush_config_queue().unwrap();
                                    analyzer
                                },
                                batch.to_owned(),
                            )
                        },
                        |(analyzer, batch)| {
                            black_box(analyzer)
                                .analyze_raw_batch(black_box(batch))
                                .unwrap()
                        },
                        BatchSize::PerIteration,
                    )
                },
            )
            .bench_with_input(
                BenchmarkId::new(
                    format!("no undef ({} files)", files.len()),
                    format!(
                        "{} thread{}",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                    ),
                ),
                &files,
                |b, batch| {
                    b.iter_batched(
                        || {
                            (
                                {
                                    let analyzer = ScopeAnalyzer::new(num_threads).unwrap();
                                    for id in (0..300).map(FileId::new) {
                                        analyzer.no_undef(id, Some(Default::default()));
                                    }

                                    analyzer.flush_config_queue().unwrap();
                                    analyzer
                                },
                                batch.to_owned(),
                            )
                        },
                        |(analyzer, batch)| {
                            black_box(analyzer)
                                .analyze_raw_batch(black_box(batch))
                                .unwrap()
                        },
                        BatchSize::PerIteration,
                    )
                },
            )
            .bench_with_input(
                BenchmarkId::new(
                    format!("no unused labels ({} files)", files.len()),
                    format!(
                        "{} thread{}",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                    ),
                ),
                &files,
                |b, batch| {
                    b.iter_batched(
                        || {
                            (
                                {
                                    let analyzer = ScopeAnalyzer::new(num_threads).unwrap();
                                    for id in (0..300).map(FileId::new) {
                                        analyzer.no_unused_labels(id, Some(Default::default()));
                                    }

                                    analyzer.flush_config_queue().unwrap();
                                    analyzer
                                },
                                batch.to_owned(),
                            )
                        },
                        |(analyzer, batch)| {
                            black_box(analyzer)
                                .analyze_raw_batch(black_box(batch))
                                .unwrap()
                        },
                        BatchSize::PerIteration,
                    )
                },
            )
            .bench_with_input(
                BenchmarkId::new(
                    format!("no unused vars ({} files)", files.len()),
                    format!(
                        "{} thread{}",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                    ),
                ),
                &files,
                |b, batch| {
                    b.iter_batched(
                        || {
                            (
                                {
                                    let analyzer = ScopeAnalyzer::new(num_threads).unwrap();
                                    for id in (0..300).map(FileId::new) {
                                        analyzer.no_unused_vars(id, Some(Default::default()));
                                    }

                                    analyzer.flush_config_queue().unwrap();
                                    analyzer
                                },
                                batch.to_owned(),
                            )
                        },
                        |(analyzer, batch)| {
                            black_box(analyzer)
                                .analyze_raw_batch(black_box(batch))
                                .unwrap()
                        },
                        BatchSize::PerIteration,
                    )
                },
            )
            .bench_with_input(
                BenchmarkId::new(
                    format!("no typeof undef ({} files)", files.len()),
                    format!(
                        "{} thread{}",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                    ),
                ),
                &files,
                |b, batch| {
                    b.iter_batched(
                        || {
                            (
                                {
                                    let analyzer = ScopeAnalyzer::new(num_threads).unwrap();
                                    for id in (0..300).map(FileId::new) {
                                        analyzer.no_typeof_undef(id, Some(Default::default()));
                                    }

                                    analyzer.flush_config_queue().unwrap();
                                    analyzer
                                },
                                batch.to_owned(),
                            )
                        },
                        |(analyzer, batch)| {
                            black_box(analyzer)
                                .analyze_raw_batch(black_box(batch))
                                .unwrap()
                        },
                        BatchSize::PerIteration,
                    )
                },
            )
            .bench_with_input(
                BenchmarkId::new(
                    format!("no use before def ({} files)", files.len()),
                    format!(
                        "{} thread{}",
                        num_threads,
                        if num_threads == 1 { "" } else { "s" },
                    ),
                ),
                &files,
                |b, batch| {
                    b.iter_batched(
                        || {
                            (
                                {
                                    let analyzer = ScopeAnalyzer::new(num_threads).unwrap();
                                    for id in (0..300).map(FileId::new) {
                                        analyzer.no_use_before_def(id, Some(Default::default()));
                                    }

                                    analyzer.flush_config_queue().unwrap();
                                    analyzer
                                },
                                batch.to_owned(),
                            )
                        },
                        |(analyzer, batch)| {
                            black_box(analyzer)
                                .analyze_raw_batch(black_box(batch))
                                .unwrap()
                        },
                        BatchSize::PerIteration,
                    )
                },
            );
    }
}

criterion_group!(
    benches,
    single_file_processing,
    multiple_file_processing,
    progressive_reprocessing_n5,
    progressive_reprocessing_n10,
    process_multiple_files_batched,
    coalescing_engine262_files,
    batched_engine262_files,
    batched_engine262_files_throughput,
    engine262_all_lints_ascending,
    engine262_all_lints_descending,
    engine262_all_lints_ascending_with_globals,
    engine262_all_lints_descending_with_globals,
    engine262_single_lints,
    initialization,
);
criterion_main!(benches);
