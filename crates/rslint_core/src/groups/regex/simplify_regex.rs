use super::maybe_parse_and_store_regex;
use crate::rule_prelude::*;
use rslint_regex::*;

macro_rules! visitor {
    ($regex:expr) => {
        #[derive(Default)]
        struct Visitor {
            ran: bool,
        }
        let mut new = Visitor::default();
        new.visit_regex($regex);
        return new.ran;
    };
}

macro_rules! run_passes {
    ($self:expr, $err:expr, $regex:expr, $($pass:ident => $msg:literal),*) => {
        // nothing to see here, just working around rustc macro bug
        #[allow(unused_parens)]
        let ($(mut $pass),*) = ($(stringify!($pass).len() == 0),*);
        loop {
            let mut new = false;
            $(
                let new_run = $self.$pass($regex);
                $pass = $pass || new_run;
                new = new || new_run;
            )*
            if !new {
                break;
            }
        }
        $(
            if $pass {
                $err = $err.footer_note($msg);
            }
        )*
    }
}

declare_lint! {
    #[derive(Default)]
    SimplifyRegex,
    regex,
    "simplify-regex"
}

#[typetag::serde]
impl CstRule for SimplifyRegex {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let (mut regex, range) = maybe_parse_and_store_regex(node, ctx.file_id)?.ok()?;
        let mut err = ctx
            .err(self.name(), "this regular expression can be simplified")
            .primary(range.clone(), "");

        run_passes! {
            self, err, &mut regex,
            remove_redundant_group_quantifiers => "`(a*)*` can be simplified to `(a*)`"
        }

        if !err.footers.is_empty() {
            let string = regex.node.to_string();
            ctx.fix().replace(range, &string);
            err.title.push_str(&format!(" to `{}`", string));
            ctx.add_err(err);
        }
        None
    }
}

impl SimplifyRegex {
    /// `(a*)*` -> `(a*)`
    pub fn remove_redundant_group_quantifiers(&self, regex: &mut Regex) -> bool {
        visitor!(regex);
        impl VisitAllMut for Visitor {
            fn visit_node(&mut self, node: &mut Node) {
                if let Node::Quantifier(_, inner, QuantifierKind::Multiple, _) = node {
                    let group = (**inner).to_owned();
                    if let Node::Group(_, Group { inner, .. }) = &mut **inner {
                        if matches!(
                            inner.expanded_nodes().last(),
                            Some(Node::Quantifier(_, _, QuantifierKind::Multiple, _))
                        ) {
                            *node = group;
                            self.ran = true;
                        }
                    }
                }
            }
        }
    }
}
