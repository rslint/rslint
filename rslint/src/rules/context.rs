//! The context a CST rule has access to when run

use codespan_reporting::diagnostic::Diagnostic;

#[derive(Debug, Clone)]
pub struct RuleContext<'ctx> {
    pub file_source: &'ctx str,
    pub file_id: &'ctx str,
    pub diagnostics: Vec<Diagnostic<&'ctx str>>,
}
