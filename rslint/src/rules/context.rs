//! The context a CST rule has access to when run

use codespan_reporting::diagnostic::Diagnostic;

#[derive(Debug, Clone)]
pub struct RuleContext<'ctx> {
    pub file_source: &'ctx str,
    pub file_id: usize,
    pub diagnostics: Vec<Diagnostic<usize>>,
}
