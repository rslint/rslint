#![cfg(test)]

mod no_undef;

use crate::{
    globals::{JsGlobal, BROWSER, BUILTIN, ES2021, NODE},
    ScopeAnalyzer,
};
use rslint_parser::parse_text;
use std::borrow::Cow;

struct DatalogTestHarness {
    datalog: ScopeAnalyzer,
    failures: Vec<String>,
    passing: usize,
}

impl DatalogTestHarness {
    pub fn new() -> Self {
        Self {
            datalog: ScopeAnalyzer::new().expect("failed to create ddlog instance"),
            failures: Vec::new(),
            passing: 0,
        }
    }

    pub fn test<C>(&mut self, code: C) -> TestCase<'_>
    where
        C: Into<Cow<'static, str>>,
    {
        TestCase::new(self, code.into())
    }

    pub fn report_outcome(self) {
        if !self.failures.is_empty() {
            panic!(
                "Test failed with {} passing, {} failing:\n{}",
                self.passing,
                self.failures.len(),
                self.failures.join("\n"),
            );
        }
    }
}

struct TestCase<'a> {
    code: Cow<'static, str>,
    globals: Vec<Cow<'static, str>>,
    browser: bool,
    node: bool,
    ecma: bool,
    invalid_name_uses: Vec<Cow<'static, str>>,
    harness: &'a mut DatalogTestHarness,
}

impl<'a> TestCase<'a> {
    pub fn new(harness: &'a mut DatalogTestHarness, code: Cow<'static, str>) -> Self {
        Self {
            code,
            globals: Vec::new(),
            browser: false,
            node: false,
            ecma: false,
            invalid_name_uses: Vec::new(),
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

    pub fn with_invalid_name_uses(mut self, invalid_name_uses: Vec<Cow<'static, str>>) -> Self {
        self.invalid_name_uses.extend(invalid_name_uses);
        self
    }

    pub fn run(mut self) {
        let mut failed = false;
        let ast = parse_text(&*self.code, 0);

        self.harness
            .datalog
            .datalog
            .inject_globals(
                &self
                    .globals
                    .iter()
                    .map(|g| JsGlobal::new(g.to_owned(), false))
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

        if self.ecma {
            self.harness
                .datalog
                .datalog
                .inject_globals(ES2021)
                .expect("failed to add ecma globals");
        }

        let mut facts = self
            .harness
            .datalog
            .analyze_inner(&ast.syntax())
            .expect("failed datalog transaction");

        for name in self.invalid_name_uses.drain(..) {
            if let Some(idx) = facts
                .invalid_name_uses
                .iter()
                .position(|n| *n.name == *name)
            {
                facts.invalid_name_uses.remove(idx);
            } else {
                self.harness
                    .failures
                    .push(format!("Missed invalid name use: {}", name));
                failed = true;
            }
        }

        if !facts.invalid_name_uses.is_empty() {
            failed = true;
            for fact in facts.invalid_name_uses.drain(..) {
                self.harness.failures.push(format!(
                    "Extra invalid name use: {} in span {}",
                    fact.name, fact.span,
                ));

                println!(
                    "failed to find `{}` within {:?}",
                    *fact.name,
                    self.harness
                        .datalog
                        .datalog
                        .variables_for_scope(fact.scope)
                        .unwrap(),
                );
            }

            panic!("{}", ast.syntax().text());
        }

        if !failed {
            self.harness.passing += 1;
        }

        // TODO: An internal refresh function?
        // if self.browser || self.node || self.ecma || !self.globals.is_empty() {
        //     self.harness
        //         .datalog
        //         .datalog
        //         .clear_globals()
        //         .expect("failed to clear ddlog globals");
        // }

        // FIXME: A wee bit of a hack around *something* going weird with
        //        scope isolation, maybe look into how scope ids are initially
        //        fabricated for new files?
        self.harness.datalog = ScopeAnalyzer::new().expect("failed to create ddlog instance");
    }
}
