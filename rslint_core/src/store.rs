//! A rule store, which houses rule groups as well as individual rules. 

use crate::{CstRuleGroup, CstRule};
use crate::groups::*;
use rayon::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct CstRuleStore {
    pub groups: Vec<CstRuleGroup>,
}

impl Default for CstRuleStore {
    fn default() -> Self {
        Self {
            groups: vec![]
        }
    }
}

impl CstRuleStore {
    pub fn new() -> Self {
        Self {
            groups: vec![]
        }
    }

    /// All built in rules from every group. 
    pub fn builtins(mut self) -> Self {
        self.groups.push(errors());
        self
    }

    /// A parallel iterator over every rule loaded into the groups of the store. 
    pub fn par_rules(&self) -> impl ParallelIterator<Item = &Box<dyn CstRule>> {
        self.groups.par_iter().map(|g| g.rules.par_iter()).flatten()
    }
    
    /// An iterator over every rule in ever group of the store. 
    pub fn rules(&self) -> impl Iterator<Item = &Box<dyn CstRule>> {
        self.groups.iter().map(|g| g.rules.iter()).flatten()
    }
}
