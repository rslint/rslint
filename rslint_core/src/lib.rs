mod rule;
mod diagnostic;
mod store;

pub mod testing;
pub mod groups;
pub mod rule_prelude;
pub mod util;

pub use codespan_reporting::diagnostic::{Label, Severity};
pub use self::{
    rule::{CstRule, RuleResult, RuleCtx, Outcome, Rule},
    diagnostic::DiagnosticBuilder,
    groups::{CstRuleGroup},
    store::CstRuleStore
};

use rayon::prelude::*;
use rslint_parser::{parse_module, SyntaxNode};

/// The type of errors, warnings, and notes emitted by the linter. 
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<usize>;

pub fn lint_file(file_id: usize, file_source: impl AsRef<str>) {

    // --------- PLACEHOLDER, used for testing the runner before a binary crate is made ---------
    use codespan_reporting::{files::SimpleFiles, term::{emit, termcolor::{StandardStream, ColorChoice}, Config}};

    let store = CstRuleStore::new().builtins();

    let parse = parse_module(file_source.as_ref(), file_id);
    let mut diagnostics = parse.errors().to_owned();
    // SyntaxNodes are not Send + Sync because they are Rc based, so we share a green node and rebuild the
    // syntax node in the closure. 
    let green = parse.green();

    let res: Vec<Diagnostic> = store.par_rules().map(|rule| {
        let root = SyntaxNode::new_root(green.clone());

        root.descendants().map(|node| {
            let mut ctx = RuleCtx {
                file_id,
                verbose: true,
                diagnostics: vec![]
            };

            rule.check_node(&node, &mut ctx);
            ctx.diagnostics
        }).flatten().collect::<Vec<_>>()
    }).flatten().collect();

    diagnostics.extend(res);

    let mut file = SimpleFiles::new();
    file.add(file_id, file_source.as_ref());
    let mut config = Config::default();
    let chars = &mut config.chars;
    chars.multi_top_left = '┌';
    chars.multi_bottom_left = '└';
    config.start_context_lines = 5;
    
    for diagnostic in diagnostics {
        emit(&mut StandardStream::stderr(ColorChoice::Always), &config, &file, &diagnostic);
    }
}

pub fn run_rule(rule: &Box<dyn CstRule>, file_id: usize, root: SyntaxNode, verbose: bool) -> Vec<Diagnostic> {
    let mut ctx = RuleCtx {
        file_id,
        verbose,
        diagnostics: vec![]
    };

    rule.check_root(&root, &mut ctx);

    root.descendants_with_tokens().for_each(|elem| {
        match elem {
            rslint_parser::NodeOrToken::Node(node) => rule.check_node(&node, &mut ctx),
            rslint_parser::NodeOrToken::Token(tok) => rule.check_token(&tok, &mut ctx)
        };
    });

    ctx.diagnostics
}

#[test]
fn placeholder() {
    let src = r#"
    Object.defineProperty(a, "key", {
        get: function() {
            switch (a > 5) {
                default:
                if (foo) {
                    return 5;
                }
            }
        }
    })
    "#;

    lint_file(0, src);
}
