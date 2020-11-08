#![cfg(test)]

mod no_undef;

use crate::{
    globals::{JsGlobal, BROWSER, BUILTIN, ES2021, NODE},
    ScopeAnalyzer,
};
use rslint_parser::{parse_module, parse_text};
use std::{
    borrow::Cow,
    fs::{self, OpenOptions},
    io::Write as _,
    ops::Range,
    path::Path,
};

struct DatalogTestHarness {
    datalog: ScopeAnalyzer,
    passing: usize,
    failing: usize,
}

impl DatalogTestHarness {
    pub fn new() -> Self {
        Self {
            datalog: ScopeAnalyzer::new().expect("failed to create ddlog instance"),
            passing: 0,
            failing: 0,
        }
    }

    pub fn test<C, R>(&mut self, code: C, rule_name: R) -> TestCase<'_>
    where
        C: Into<Cow<'static, str>>,
        R: Into<Cow<'static, str>>,
    {
        TestCase::new(self, code.into(), rule_name.into())
    }

    pub fn report_outcome(self) {
        if self.failing != 0 {
            panic!(
                "Test failed with {} passing, {} failing, logs stored in `{}/output.log/`",
                self.passing,
                self.failing,
                env!("CARGO_MANIFEST_DIR"),
            );
        }
    }
}

struct TestCase<'a> {
    rule_name: Cow<'static, str>,
    code: Cow<'static, str>,
    globals: Vec<Cow<'static, str>>,
    browser: bool,
    node: bool,
    ecma: bool,
    is_module: bool,
    es2021: bool,
    invalid_name_uses: Vec<(Cow<'static, str>, Range<u32>)>,
    harness: &'a mut DatalogTestHarness,
    counter: usize,
}

impl<'a> TestCase<'a> {
    pub fn new(
        harness: &'a mut DatalogTestHarness,
        code: Cow<'static, str>,
        rule_name: Cow<'static, str>,
    ) -> Self {
        Self {
            rule_name,
            code,
            globals: Vec::new(),
            browser: false,
            node: false,
            ecma: false,
            is_module: false,
            es2021: false,
            invalid_name_uses: Vec::new(),
            harness,
            counter: 0,
        }
    }

    pub fn with_globals(mut self, globals: Vec<Cow<'static, str>>) -> Self {
        self.globals.extend(globals);
        self
    }

    pub fn with_browser(mut self, browser: bool) -> Self {
        self.browser = browser;
        self
    }

    pub fn with_node(mut self, node: bool) -> Self {
        self.node = node;
        self
    }

    pub fn with_ecma(mut self, ecma: bool) -> Self {
        self.ecma = ecma;
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_module(mut self, is_module: bool) -> Self {
        self.is_module = is_module;
        self
    }

    pub fn with_es2021(mut self, es2021: bool) -> Self {
        self.es2021 = es2021;
        self
    }

    pub fn with_invalid_name_uses(
        mut self,
        invalid_name_uses: Vec<(Cow<'static, str>, Range<u32>)>,
    ) -> Self {
        self.invalid_name_uses.extend(invalid_name_uses);
        self
    }

    // TODO: This is so ugly
    pub fn run(mut self) {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/output.log"));
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }

        self.harness.datalog.with_replay_file(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join(&format!("{}-{}.replay", self.rule_name, self.counter)))
                .unwrap(),
        );
        self.harness.datalog.outputs().with_output_file(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join(&format!("{}-{}.state", self.rule_name, self.counter)))
                .unwrap(),
        );

        let mut failed = false;
        let ast = if self.is_module {
            parse_module(&*self.code, 0).syntax()
        } else {
            parse_text(&*self.code, 0).syntax()
        };

        self.harness
            .datalog
            .datalog
            .inject_globals(
                &self
                    .globals
                    .iter()
                    .map(|g| JsGlobal::new(g.to_string(), false))
                    .collect::<Vec<_>>(),
            )
            .expect("failed to inject global variables");

        self.harness
            .datalog
            .datalog
            .inject_globals(BUILTIN)
            .expect("failed to inject builtin variables");

        if self.browser {
            self.harness
                .datalog
                .datalog
                .inject_globals(BROWSER)
                .expect("failed to add browser globals");
        }

        if self.node {
            self.harness
                .datalog
                .datalog
                .inject_globals(NODE)
                .expect("failed to add node globals");
        }

        if self.ecma || self.es2021 {
            self.harness
                .datalog
                .datalog
                .inject_globals(ES2021)
                .expect("failed to add ecma globals");
        }

        self.harness
            .datalog
            .analyze_inner(&ast)
            .expect("failed datalog transaction");
        let mut facts: Vec<_> = self
            .harness
            .datalog
            .outputs()
            .invalid_name_use
            .iter()
            .map(|usage| usage.key().clone())
            .collect();
        let uses = self.invalid_name_uses.clone();

        for (name, range) in self.invalid_name_uses.drain(..) {
            if let Some(idx) = facts
                .iter()
                .position(|n| *n.name == *name && n.span == range.clone().into())
            {
                facts.remove(idx);
            } else {
                failed = true;
            }
        }

        if failed || !facts.is_empty() {
            self.harness.failing += 1;

            let mut file = OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(&format!(
                    "{}/output.log/{}-{}.failure",
                    env!("CARGO_MANIFEST_DIR"),
                    self.rule_name,
                    self.counter,
                ))
                .unwrap();

            write!(
                &mut file,
                "============ FAILURE ============\n\n\
                => Source:\n{}\n\n\
                => Expected:\n{}\n\n\
                => Got:\n{}\n\n\
                => Inputs:\n{}\n\n\
                => Outputs:\n{:#?}\n\n\
                ============ END FAILURE ============\n\n",
                ast.text(),
                uses.into_iter()
                    .map(|(name, range)| format!("  {:?} @ {:?}", name, range))
                    .collect::<Vec<_>>()
                    .join("\n"),
                self.harness
                    .datalog
                    .outputs()
                    .invalid_name_use
                    .iter()
                    .map(|usage| {
                        if facts.contains(usage.key()) {
                            format!("+ {:?}", usage.key())
                        } else {
                            format!("  {:?}", usage.key())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                self.harness.datalog.dump_inputs().unwrap(),
                self.harness.datalog.outputs(),
            )
            .unwrap();
        } else {
            self.harness.passing += 1;
        }

        self.harness.datalog.datalog.reset().unwrap();
        self.counter += 1;
    }
}
