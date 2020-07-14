use crate::register_errors_group;
use crate::rules::{CstRule, CstRuleGroup};

pub struct CstRuleStore {
    pub groups: Vec<CstRuleGroup>,
}

impl CstRuleStore {
    pub fn new() -> Self {
        Self { groups: vec![] }
    }

    /// Load builtin groups from `/rules`
    pub fn load_predefined_groups(mut self) -> Self {
        let groups = &mut self.groups;
        register_errors_group!(groups);
        self
    }

    /// Insert a single rule into the store, if a group with that name exists it will be inserted into that group, or else a new group will be created
    pub fn load_rule(&mut self, group_name: &'static str, rule: Box<dyn CstRule>) {
        let existing = self.groups.iter_mut().find(|group| group.name == group_name);

        if let Some(group) = existing {
            group.rules.push(rule);
        } else {
            self.groups.push(CstRuleGroup::new(group_name, vec![rule]));
        }
    }
}
