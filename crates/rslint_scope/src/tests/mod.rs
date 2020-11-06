#![cfg(test)]

mod no_undef;

use crate::{
    globals::{JsGlobal, BROWSER, BUILTIN, ES2021, NODE},
    ScopeAnalyzer,
};
use rslint_parser::{parse_module, parse_text};
use std::{borrow::Cow, fmt::Write};

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

    pub fn test<C, R>(&mut self, code: C, rule_name: R) -> TestCase<'_>
    where
        C: Into<Cow<'static, str>>,
        R: Into<Cow<'static, str>>,
    {
        TestCase::new(self, code.into(), rule_name.into())
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
    rule_name: Cow<'static, str>,
    code: Cow<'static, str>,
    globals: Vec<Cow<'static, str>>,
    browser: bool,
    node: bool,
    ecma: bool,
    is_module: bool,
    es2021: bool,
    invalid_name_uses: Vec<Cow<'static, str>>,
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

    #[allow(clippy::wrong_self_convention)]
    pub fn is_module(mut self, is_module: bool) -> Self {
        self.is_module = is_module;
        self
    }

    pub fn with_es2021(mut self, es2021: bool) -> Self {
        self.es2021 = es2021;
        self
    }

    pub fn with_invalid_name_uses(mut self, invalid_name_uses: Vec<Cow<'static, str>>) -> Self {
        self.invalid_name_uses.extend(invalid_name_uses);
        self
    }

    // TODO: This is so ugly
    pub fn run(mut self) {
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
        let mut facts = self
            .harness
            .datalog
            .datalog
            .invalid_name_uses(None)
            .unwrap();

        for name in self.invalid_name_uses.drain(..) {
            if let Some(idx) = facts.iter().position(|n| *n.name == *name) {
                facts.remove(idx);
            } else {
                // FIXME: Make this detailed
                self.harness
                    .failures
                    .push(format!("Missed invalid name use: {}", name));
                failed = true;
            }
        }

        if !facts.is_empty() {
            failed = true;

            for fact in facts.drain(..) {
                let mut error = String::new();

                let mut vars = self
                    .harness
                    .datalog
                    .datalog
                    .variables_for_scope(Some(fact.scope))
                    .unwrap();
                vars.sort();

                // FIXME: This is ugly as hell
                write!(
                    &mut error,
                    "- failed to find `{}`\n  Input: `{}`\n  Span: {:?}  \n  Scope: [\n{}\n  ]\n",
                    *fact.name,
                    ast.text(),
                    fact.span,
                    vars.chunks(6)
                        .map(|chunk| "      ".to_string()
                            + &chunk
                                .iter()
                                .map(|s| s.name.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                            + ",")
                        .collect::<Vec<_>>()
                        .join("\n")
                )
                .unwrap();

                self.harness.failures.push(format!(
                    "failed to find `{}`\n  Input: `{}`\n",
                    *fact.name,
                    ast.text(),
                ));
            }
        }

        if !failed {
            self.harness.passing += 1;
        }

        self.harness.datalog.datalog.reset().unwrap();
    }
}
