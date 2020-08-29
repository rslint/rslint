pub mod errors;

use crate::CstRule;
use std::cmp::{Eq, PartialEq};

pub use errors::errors;

/// A group of CST rules with a common scope.
/// Each group is identified by a static string which uses a hashmap
/// which maps static strings to a rule.
#[derive(Debug)]
pub struct CstRuleGroup {
    pub rules: Vec<Box<dyn CstRule>>,
    pub name: &'static str,
}

impl CstRuleGroup {
    pub fn new(name: &'static str) -> Self {
        Self {
            rules: vec![],
            name,
        }
    }

    /// Load a rule into the group.
    pub fn load_rule(&mut self, rule: Box<dyn CstRule>) {
        self.rules.push(rule);
    }
}

impl PartialEq for CstRuleGroup {
    fn eq(&self, other: &CstRuleGroup) -> bool {
        // We assume keys are unique and they map to the right rule. This invariant
        // is upheld throughout the linter and failure to do so is the user's fault.
        self.name == other.name
            && self
                .rules
                .iter()
                .map(|x| x.name())
                .eq(other.rules.iter().map(|x| x.name()))
    }
}
impl Eq for CstRuleGroup {}

/// Macro for easily making a rule group hashmap.
/// This will call `::new()` on each rule.  
#[macro_export]
macro_rules! group {
    ($groupname:ident, $($path:ident::$rule:ident),*) => {
        use $crate::{CstRule, CstRuleGroup};
        $(
            mod $path;
            pub use $path::$rule;
        )*

        pub fn $groupname() -> CstRuleGroup {
            CstRuleGroup {
                rules: vec![$(Box::new($rule::new()) as Box<dyn CstRule>),*],
                name: &stringify!($groupname)
            }
        }
    };
}
