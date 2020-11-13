#![cfg(test)]

macro_rules! rule_test {
    (
        $rule_name:ident,
        $(filter: $filter:expr,)?
        $({
            $($code:literal),+
            $(, globals: [$($global:literal),* $(,)?])?
            $(, browser: $browser:literal)?
            $(, node: $node:literal)?
            $(, ecma: $ecma:literal)?
            $(, module: $module:literal)?
            $(, es2021: $es2021:literal)?
            $(, errors: [$($error:expr),* $(,)?])?
            $(,)?
        }),* $(,)?
    ) => {
        #[test]
        fn $rule_name() {
            #[allow(unused_imports)]
            use crate::{tests::DatalogTestHarness, datalog::DatalogLint::{self, *}};
            #[allow(unused_imports)]
            use types::ast::Span;
            #[allow(unused_imports)]
            use std::borrow::Cow;

            let mut analyzer = DatalogTestHarness::new()
                $(.with_filter($filter as fn(&DatalogLint) -> bool))?;

            $(
                analyzer
                    .test(vec![$($code,)+].join("\n"), stringify!($rule_name))
                    $(.with_globals(vec![$(Cow::Borrowed($global)),*]))?
                    $(.with_browser($browser))?
                    $(.with_node($node))?
                    $(.with_ecma($ecma))?
                    $(.is_module($module))?
                    $(.with_es2021($es2021))?
                    $(.with_errors(vec![$($error),*]))?
                    .run();
            )?

            analyzer.report_outcome();
        }
    };
}

mod no_undef;
mod no_unused_vars;

use crate::{
    datalog::DatalogLint,
    globals::{JsGlobal, BROWSER, BUILTIN, ES2021, NODE},
    ScopeAnalyzer,
};
use rslint_parser::{parse_module, parse_text};
use std::{
    borrow::Cow,
    fs::{self, OpenOptions},
    io::Write as _,
    path::Path,
};
use types::ast::FileId;

struct DatalogTestHarness {
    datalog: ScopeAnalyzer,
    passing: usize,
    failing: usize,
    counter: usize,
    filter: Option<fn(&DatalogLint) -> bool>,
}

impl DatalogTestHarness {
    pub fn new() -> Self {
        Self {
            datalog: ScopeAnalyzer::new().expect("failed to create ddlog instance"),
            passing: 0,
            failing: 0,
            counter: 0,
            filter: None,
        }
    }

    pub fn with_filter(mut self, filter: fn(&DatalogLint) -> bool) -> Self {
        self.filter = Some(filter);
        self
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
    errors: Vec<DatalogLint>,
    harness: &'a mut DatalogTestHarness,
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
            errors: Vec::new(),
            harness,
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

    pub fn with_errors(mut self, errors: Vec<DatalogLint>) -> Self {
        self.errors.extend(errors);
        self
    }

    // TODO: This is so ugly
    pub fn run(mut self) {
        let file_id = FileId::new(self.harness.counter as u32);

        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/output.log"))
            .join(format!("{}-{}", self.rule_name, self.harness.counter));
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }

        self.harness.datalog.with_replay_file(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join("replay"))
                .unwrap(),
        );
        self.harness.datalog.outputs().with_output_file(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join("state"))
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
                file_id,
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
            .inject_globals(file_id, BUILTIN)
            .expect("failed to inject builtin variables");

        if self.browser {
            self.harness
                .datalog
                .datalog
                .inject_globals(file_id, BROWSER)
                .expect("failed to add browser globals");
        }

        if self.node {
            self.harness
                .datalog
                .datalog
                .inject_globals(file_id, NODE)
                .expect("failed to add node globals");
        }

        if self.ecma || self.es2021 {
            self.harness
                .datalog
                .datalog
                .inject_globals(file_id, ES2021)
                .expect("failed to add ecma globals");
        }

        self.harness
            .datalog
            .analyze(file_id, &ast)
            .expect("failed datalog transaction");

        let mut errors = self.harness.datalog.get_lints().unwrap();
        if let Some(filter) = self.harness.filter {
            errors = errors.into_iter().filter(filter).collect();
        }

        for error in self.errors.iter() {
            if let Some(idx) = errors.iter().position(|err| err == error) {
                errors.remove(idx);
            } else {
                failed = true;
            }
        }

        if failed || !errors.is_empty() {
            self.harness.failing += 1;

            let mut file = OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(path.join("failure"))
                .unwrap();

            write!(
                &mut file,
                "============ FAILURE ============\n\n\
                => Source:\n{}\n\n\
                => Expected:\n{:#?}\n\n\
                => Got:\n{:#?}\n\n\
                => Inputs:\n{}\n\n\
                => Outputs:\n{:#?}\n\n\
                ============ END FAILURE ============\n\n",
                ast.text(),
                self.errors,
                self.harness.datalog.get_lints().unwrap(),
                self.harness.datalog.dump_inputs().unwrap(),
                self.harness.datalog.outputs(),
            )
            .unwrap();
        } else {
            fs::remove_dir_all(&path).unwrap();
            self.harness.passing += 1;
        }

        self.harness.datalog.datalog.reset().unwrap();
        self.harness.counter += 1;
    }
}
