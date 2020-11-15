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
        self
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
}

/*
use crate::rule_prelude::*;
use rslint_scope::{DatalogLint, FileId, ScopeAnalyzer};

declare_lint! {
    /**
    Disallow undefined variables
    */
    #[derive(Default)]
    Scoper,
    errors,
    "scoper",

    pub analyzer: ScopeAnalyzer,
}

#[typetag::serde]
impl CstRule for Scoper {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        for lint in self
            .analyzer
            .get_lints(FileId::new(ctx.file_id as u32))
            .unwrap()
        {
            let err = match lint {
                DatalogLint::NoUndef { var, span, .. } => ctx
                    .err("no-undef", "a variable was used, but never defined")
                    .label(
                        Severity::Error,
                        span,
                        format!("`{}` was used, but never defined", *var),
                    ),

                DatalogLint::NoUnusedVars { var, declared, .. } => {
                    ctx.err("no-unused-vars", "a variable is never used").label(
                        Severity::Warning,
                        declared,
                        format!("`{}` is never used", *var),
                    )
                }

                DatalogLint::TypeofUndef {
                    whole_expr,
                    undefined_portion,
                    ..
                } => ctx
                    .err(
                        "typeof-undef",
                        "calling `typeof` un an undefined variable will always result in `undefined`",
                    )
                    .primary(whole_expr, "`typeof` is called here")
                    .secondary(undefined_portion, "this expression is undefined")
                    .suggestion(
                        whole_expr,
                        "if this is intentional, replace this expression with `undefined`",
                        "undefined",
                        Applicability::MaybeIncorrect,
                    ),

                DatalogLint::UseBeforeDef {
                    name,
                    used,
                    declared,
                    ..
                } => ctx
                    .err(
                        "no-use-before-def",
                        format!("`{}` was used before it was defined", *name),
                    )
                    .primary(declared, format!("`{}` was defined here", *name))
                    .secondary(used, "but used here"),

                DatalogLint::NoShadow { variable, original, shadow, implicit, .. } => {
                    ctx
                        .err(
                            "no-shadow",
                            if implicit { "an implicit variable was shadowed" } else { "a variable was shadowed" },
                        )
                        .primary(original, format!("`{}` was originally declared here", *variable))
                        .secondary(shadow, "and shadowed here")
                }

                DatalogLint::NoUnusedLabels { label, span, .. } => {
                     ctx
                        .err(
                            "no-unused-labels",
                            "a label was created, but never used"
                        )
                        .primary(span, format!("`{}` was created here and never used", *label))
                }
            };

            ctx.add_err(err);
        }

        None
    }
}
*/
