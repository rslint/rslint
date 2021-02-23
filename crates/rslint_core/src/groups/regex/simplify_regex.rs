use super::maybe_parse_and_store_regex;
use crate::rule_prelude::*;
use rslint_regex::Span;
use rslint_regex::*;

macro_rules! visitor {
    ($regex:expr, $src:expr) => {
        struct Visitor<'a> {
            ran: bool,
            #[allow(dead_code)]
            src: &'a str,
        }
        let mut new = Visitor {
            ran: false,
            src: $src,
        };
        new.visit_regex($regex);
        return new.ran;
    };
}

macro_rules! run_passes {
    ($self:expr, $err:expr, $regex:expr, $src:expr, $($pass:ident => $msg:literal),* $(,)?) => {
        // nothing to see here, just working around rustc macro bug
        #[allow(unused_parens)]
        let ($(mut $pass),*) = ($(stringify!($pass).len() == 0),*);
        loop {
            let mut new = false;
            $(
                let new_run = $self.$pass($regex, $src);
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
    /**
    Simplify regular expressions.

    RegEx can oftentimes be simplified into smaller and more idiomatic expressions.
    This leads to more readable code by reducing the complexity of the expressions.

    This rule attempts to recursively simplify regular expressions and offer an autofix for it.
    */
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

        let src = ctx.src.as_ref();

        run_passes! {
            self, err, &mut regex, src,
            remove_redundant_group_quantifiers => "`(a*)*` can be simplified to `(a*)`",
            // this pass should run before `[0-9]` -> `\d` so that it tries to simplify the longest sequence first
            // without simplifying `[a-zA-Z0-9_]` to `[a-zA-Z\d_]`
            use_explicit_word_escape => r"`[a-zA-Z0-9_]` is equivalent to `\w`",
            use_explicit_digit_escape => r"`[0-9]` is equivalent to `\d`",
            use_explicit_star_quantifier => r"`a{0,}` is equivalent to `a*`",
            use_explicit_plus_quantifier => r"`a{1,}` is equivalent to `a+`",
            collapse_contiguous_literals => "contiguous literals can be turned into a quantifier"
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
    pub fn remove_redundant_group_quantifiers(&self, regex: &mut Regex, src: &str) -> bool {
        visitor!(regex, src);
        impl VisitAllMut for Visitor<'_> {
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

    /// `[0-9]` -> `\d` and `[a0-9]` -> `[a\d]`
    pub fn use_explicit_digit_escape(&self, regex: &mut Regex, src: &str) -> bool {
        visitor!(regex, src);
        impl VisitAllMut for Visitor<'_> {
            fn visit_node(&mut self, node: &mut Node) {
                if let Node::CharacterClass(_, class) = node {
                    let contains = class
                        .members
                        .iter()
                        .any(|member| member.is(self.src, "0-9"));
                    if contains {
                        self.ran = true;
                        if class.members.len() == 1 {
                            *node = Node::from_string("\\d").unwrap();
                        } else {
                            let element = class
                                .members
                                .iter_mut()
                                .find(|member| member.is(self.src, "0-9"))
                                .unwrap();
                            *element =
                                CharacterClassMember::Single(Node::from_string("\\d").unwrap());
                        }
                    }
                }
            }
        }
    }

    /// `[0-9]` -> `\d` and `[a0-9]` -> `[a\d]`
    pub fn use_explicit_word_escape(&self, regex: &mut Regex, src: &str) -> bool {
        visitor!(regex, src);
        impl VisitAllMut for Visitor<'_> {
            fn visit_node(&mut self, node: &mut Node) {
                if let Node::CharacterClass(_, class) = node {
                    let (mut a_to_z, mut a_to_z_cap, mut zero_to_nine, mut underscore) =
                        (None, None, None, None);
                    for member in &mut class.members {
                        if member.is(self.src, "a-z") {
                            a_to_z = Some(member);
                        } else if member.is(self.src, "A-Z") {
                            a_to_z_cap = Some(member);
                        } else if member.is(self.src, "0-9") {
                            zero_to_nine = Some(member);
                        } else if member.is(self.src, "_") {
                            underscore = Some(member);
                        }
                    }
                    if let (Some(a_to_z), Some(a_to_z_cap), Some(zero_to_nine), Some(underscore)) =
                        (a_to_z, a_to_z_cap, zero_to_nine, underscore)
                    {
                        self.ran = true;
                        let new = CharacterClassMember::Single(Node::Empty);
                        *a_to_z = new.clone();
                        *a_to_z_cap = new.clone();
                        *zero_to_nine = new;
                        *underscore =
                            CharacterClassMember::Single(Node::from_string("\\w").unwrap());
                    }
                }
            }
        }
    }

    /// `a{0,}` -> `a*`
    pub fn use_explicit_star_quantifier(&self, regex: &mut Regex, src: &str) -> bool {
        visitor!(regex, src);
        impl VisitAllMut for Visitor<'_> {
            fn visit_quantifier(
                &mut self,
                _: &Span,
                _: &mut Node,
                kind: &mut QuantifierKind,
                _: &mut bool,
            ) {
                if *kind == QuantifierKind::Between(0, None) {
                    *kind = QuantifierKind::Multiple;
                    self.ran = true;
                }
            }
        }
    }

    /// `a{1,}` -> `a+`
    pub fn use_explicit_plus_quantifier(&self, regex: &mut Regex, src: &str) -> bool {
        visitor!(regex, src);
        impl VisitAllMut for Visitor<'_> {
            fn visit_quantifier(
                &mut self,
                _: &Span,
                _: &mut Node,
                kind: &mut QuantifierKind,
                _: &mut bool,
            ) {
                if *kind == QuantifierKind::Between(1, None) {
                    *kind = QuantifierKind::AtLeastOne;
                    self.ran = true;
                }
            }
        }
    }

    /// `aaa` -> `a{3}`
    pub fn collapse_contiguous_literals(&self, regex: &mut Regex, src: &str) -> bool {
        visitor!(regex, src);
        impl VisitAllMut for Visitor<'_> {
            fn visit_node(&mut self, node: &mut Node) {
                if let Node::Alternative(_, nodes) = node {
                    let mut iter = nodes.iter_mut().peekable();
                    while let Some(node) = iter.next() {
                        if let Node::Literal(_, c, _) = node {
                            let mut number = 0;
                            let mut node_stack = vec![];
                            while let Some(next) = iter.peek() {
                                match next {
                                    Node::Literal(_, next_c, _) if next_c == c => {
                                        node_stack.push(iter.next().unwrap());
                                        number += 1;
                                    }
                                    _ => break,
                                }
                            }
                            if number != 0 {
                                for node in node_stack {
                                    *node = Node::Empty;
                                }
                                *node = Node::Quantifier(
                                    node.span().unwrap().to_owned(),
                                    Box::new(node.to_owned()),
                                    QuantifierKind::Number(number as u32 + 1),
                                    false,
                                );
                                self.ran = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
