//! A rule store, which houses rule groups as well as individual rules. 

use crate::CstRule;
use crate::groups::*;

#[derive(Debug, Default)]
pub struct CstRuleStore {
    pub rules: Vec<Box<dyn CstRule>>,
}

impl CstRuleStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// All built in rules from every group. 
    pub fn builtins(mut self) -> Self {
        self.rules.extend(errors());
        self
    }

    pub fn load_rules(&mut self, rules: impl IntoIterator<Item = Box<dyn CstRule>>) {
        self.rules.extend(rules);
    }
}
