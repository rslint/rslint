//! Rules which are concurrently run on parsed JavaScript files.

#![allow(unused_variables)]

pub mod context;
pub mod store;
pub mod groups;

use codespan_reporting::diagnostic::{Severity, Diagnostic};
use self::context::RuleContext;
use std::fmt::{Formatter, Debug};
use std::hash::{Hash, Hasher};
use rslint_parse::parser::cst::*;

/// A struct representing a group of cst rules
pub struct CstRuleGroup {
    pub name: &'static str,
    pub rules: Vec<Box<dyn CstRule>>,
}

impl CstRuleGroup {
    pub fn new(name: &'static str, rules: Vec<Box<dyn CstRule>>) -> Self {
        Self { name, rules }
    }
}

/// A macro for easily creating rules, consisting of the name of the rule as a string, and its struct name
/// After using this you must import the Visit trait and implement it for the struct
#[macro_export]
macro_rules! cst_rule {
    ($name:expr, $struct_name:ident) => {
        use crate::rules::CstRule;
        use crate::rules::context::RuleContext;
        use rslint_parse::parser::cst::CST;

        #[derive(Debug)]
        pub struct $struct_name;

        paste::item! {
            #[derive(Debug)]
            pub struct [<$struct_name Visitor>]<'a, 'b> {
                pub ctx: &'a mut RuleContext<'b>
            }
        }

        impl CstRule for $struct_name {
            fn name(&self) -> &'static str {
                $name
            }
            
            paste::item! {
                fn lint(&self, ctx: &mut RuleContext, cst: &CST) {
                    let mut visitor = [<$struct_name Visitor>] { ctx };
                    visitor.visit_cst(cst, cst);
                }
            }
        }
    }
}

/// A trait describing a rule which is run on a single file and its CST (concrete syntax tree)
pub trait CstRule: Send + Sync {
    /// Get the name of the rule, **this must be unique across groups to prevent conflicts**
    fn name(&self) -> &'static str;

    /// Run the rule on a context and report results by adding any diagnostics to it
    fn lint(&self, ctx: &mut RuleContext, cst: &CST);
}

impl PartialEq for dyn CstRule {
    fn eq(&self, other: &dyn CstRule) -> bool {
        self.name() == other.name()
    }
}

impl Eq for dyn CstRule {}

impl Debug for dyn CstRule {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Hash for dyn CstRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name().as_bytes())
    }
}

/// The outcome of running a rule on a single file
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Outcome {
    Success,
    Error,
    Warning,
}

/// The overall result of running a rule on a file constructed by a vector of diagnostics
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub outcome: Outcome,
    pub diagnostics: Vec<Diagnostic<usize>>,
}

impl RuleResult {
    /// Make a new rule result indicating that the rule was a success
    pub fn success() -> RuleResult {
        Self {
            outcome: Outcome::Success,
            diagnostics: vec![],
        }
    }

    /// Make a new result indicating an error status
    pub fn error(diagnostic: impl Into<Diagnostic<usize>>) -> RuleResult {
        Self {
            outcome: Outcome::Error,
            diagnostics: vec![diagnostic.into()]
        }
    }

    /// Merge multiple results by taking all the diagnostics and joining them, and making the outcome the worst of the results  
    /// e.g. Success + Warn => Warn, Success + Warn + Error => Error
    pub fn merge(results: Vec<Self>) -> Self {
        let mut merged = Self::success();
        for result in results.into_iter() {
            if let Outcome::Success = result.outcome {
                merged.diagnostics.extend(result.diagnostics);
            } else {
                match result.outcome {
                    Outcome::Warning if merged.outcome == Outcome::Success => merged.outcome = Outcome::Warning,
                    Outcome::Error => merged.outcome = Outcome::Error,
                    _ => unreachable!(),
                }
                merged.diagnostics.extend(result.diagnostics);
            }
        }
        merged
    }
}

/// Make a new rule result from a list of diagnostics
impl<'a> From<Vec<Diagnostic<usize>>> for RuleResult {
    fn from(diagnostics: Vec<Diagnostic<usize>>) -> Self {
        let mut outcome = Outcome::Success;
        for diagnostic in diagnostics.iter() {
            match diagnostic.severity {
                Severity::Error | Severity::Bug => outcome = Outcome::Error,
                Severity::Warning => outcome = Outcome::Warning,
                _ => {},
            }
        }

        Self {
            outcome,
            diagnostics
        }
    }
}

/// Allow merging two results by adding them with `+`
impl std::ops::Add for RuleResult {
    type Output = RuleResult;

    fn add(self, other: RuleResult) -> RuleResult {
        RuleResult::merge(vec![self, other])
    }
}

impl std::ops::AddAssign for RuleResult {
    fn add_assign(&mut self, other: RuleResult) {
        if let Outcome::Success = other.outcome {
            self.diagnostics.extend(other.diagnostics);
        } else {
            match other.outcome {
                Outcome::Warning if self.outcome == Outcome::Success => self.outcome = Outcome::Warning,
                Outcome::Error => self.outcome = Outcome::Error,
                _ => unreachable!(),
            }
            self.diagnostics.extend(other.diagnostics);
        }
    }
}