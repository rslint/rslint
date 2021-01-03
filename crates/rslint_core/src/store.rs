//! A rule store, which houses rule groups as well as individual rules.

use crate::groups::*;
use crate::CstRule;

/// A utility structure for housing CST rules for a linting run.
#[derive(Debug, Default, Clone)]
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
        self.rules.extend(style());
        self.rules.extend(ddlog());
        self
    }

    pub fn ddlog() -> Self {
        let mut this = Self::new();
        this.rules.extend(ddlog());
        this
    }

    /// Load a single rule into this store.
    pub fn load_rule(&mut self, rule: Box<dyn CstRule>) {
        self.rules.push(rule);
    }

    /// Load a list of rules into this store.
    pub fn load_rules(&mut self, rules: impl IntoIterator<Item = Box<dyn CstRule>>) {
        self.rules.extend(rules);
    }

    /// Get a rule using its rule name from this store.
    ///
    /// # Examples
    /// ```
    /// use rslint_core::CstRuleStore;
    ///
    /// assert!(CstRuleStore::new().builtins().get("no-empty").is_some())
    /// ```
    pub fn get(&self, rule_name: impl AsRef<str>) -> Option<Box<dyn CstRule>> {
        self.rules
            .iter()
            .find(|rule| rule.name() == rule_name.as_ref())
            .cloned()
    }

    /// Returns whether or not the store contains the given rule
    pub fn contains(&self, rule_name: impl AsRef<str>) -> bool {
        self.rules
            .iter()
            .any(|rule| rule.name() == rule_name.as_ref())
    }

    /// Returns the number of currently loaded rules
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns whether the rule store is empty or not
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes the rules where `filter` returns `true`
    pub fn filter<F>(&mut self, mut filter: F)
    where
        F: FnMut(&dyn CstRule) -> bool,
    {
        // TODO: Replace with `Vec::drain_filter()`
        let mut i = 0;
        while i != self.rules.len() {
            if filter(&*self.rules[i]) {
                self.rules.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
