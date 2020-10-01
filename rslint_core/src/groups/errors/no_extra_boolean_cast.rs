use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
     * In contexts where expression will be coerced to a `Boolean` (e.g. `if`),
     * casting to a boolean (using `!!` or `Boolean(expr)`) is unnecessary.
     *
     * # Invalid Code Examples
     * ```ignore
     * if (!!foo) {}
     * while (!!foo) {}
     *
     * var foo = !!!bar;
     * var foo = Boolean(!!bar);
     * ```
     */
    #[derive(Default)]
    #[serde(default)]
    NoExtraBooleanCast,
    errors,
    "no-extra-boolean-cast",

    /**
     * If this option is `true`, this rule will also check for unnecessary boolean
     * cast inside logical expression, which is disabled by default.
     */
    pub enforce_for_logical_operands: bool,
}

const BOOL_NODE_KINDS: [SyntaxKind; 5] = [IF_STMT, DO_WHILE_STMT, WHILE_STMT, COND_EXPR, FOR_STMT];

/// The reason the cast is not needed
#[derive(Debug)]
enum Reason {
    ExplicitBoolean(SyntaxNode),
    ImplicitCast(SyntaxNode),
    LogicalNotCast(SyntaxToken),
}

#[typetag::serde]
impl CstRule for NoExtraBooleanCast {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        match node.kind() {
            UNARY_EXPR => {
                let expr = node.to::<UnaryExpr>();
                let child = skip_grouping_child(node.clone())?.try_to::<Expr>()?;

                if expr.op()? != op![!]
                    || !matches!(child, Expr::UnaryExpr(expr) if expr.op()? == op![!])
                {
                    return None;
                }

                if let Some(reason) = in_bool_ctx(node) {
                    let err = ctx.err(self.name(), "redundant double negation").primary(
                        expr.op_token().unwrap().text_range(),
                        "this operator is redundant...",
                    );
                    ctx.add_err(reason_labels(err, reason));
                }
            }
            CALL_EXPR => {
                if !util::constructor_or_call_with_callee(node, "Boolean") {
                    return None;
                }

                if let Some(reason) = in_bool_ctx(node) {
                    let err = ctx.err(self.name(), "redundant `Boolean` call").primary(
                        node.trimmed_range(),
                        "this call to `Boolean` is redundant...",
                    );
                    ctx.add_err(reason_labels(err, reason));
                }
            }
            _ => {}
        }
        None
    }
}

fn reason_labels(builder: DiagnosticBuilder, reason: Reason) -> DiagnosticBuilder {
    match reason {
        Reason::ExplicitBoolean(node) => builder.secondary(
            node.trimmed_range(),
            "...because `Boolean` already creates a boolean value",
        ),
        Reason::ImplicitCast(node) => builder.secondary(
            node.trimmed_range(),
            "...because this condition already implicitly coerces to a boolean",
        ),
        Reason::LogicalNotCast(token) => builder.secondary(
            token.text_range(),
            "...because this operator already coerces to a boolean",
        ),
    }
}

fn in_bool_ctx(node: &SyntaxNode) -> Option<Reason> {
    let parent = skip_grouping_parent(node.clone()).and_then(|node| node.parent());
    if let Some(parent) = parent {
        // TODO: Once we have scope analysis we can know if Boolean was shadowed
        // new Boolean(foo) or Boolean(foo)
        if util::constructor_or_call_with_callee(parent.clone(), "Boolean") {
            return parent
                .child_with_kind(ARG_LIST)
                .filter(|cond| skip_grouping_child(cond.clone()).as_ref() == Some(node))
                .map(|_| Reason::ExplicitBoolean(parent));
        }
    }

    if let Some(casted_node) = implicitly_casted_node(node) {
        let cond_node = match casted_node.kind() {
            IF_STMT | DO_WHILE_STMT | WHILE_STMT => casted_node
                .child_with_kind(CONDITION)
                .and_then(|n| n.first_child()),
            FOR_STMT => casted_node.child_with_kind(FOR_STMT_TEST)?.first_child(),
            COND_EXPR => casted_node
                .to::<CondExpr>()
                .test()
                .map(|x| x.syntax().clone()),
            _ => None,
        };

        return cond_node
            .filter(|inner| {
                let mut iter = std::iter::successors(Some(inner.clone()), |last| last.parent())
                    .skip_while(|node| node.kind() == GROUPING_EXPR);
                iter.next().as_ref() == Some(node)
            })
            .map(|node| Reason::ImplicitCast(node));
    }

    let parent = skip_grouping_parent(node.clone())?;
    if let Some(unexpr) = parent.try_to::<UnaryExpr>() {
        let (tok, op) = unexpr.op_details()?;
        if op == op![!] {
            return Some(Reason::LogicalNotCast(tok));
        }
    }

    None
}

// TODO: I'm pretty sure this is not the right way to make
// `Boolean(!(!(a, b)))` and others work.
fn skip_grouping_child(mut child: SyntaxNode) -> Option<SyntaxNode> {
    loop {
        child = child.first_child()?;
        if child.kind() == GROUPING_EXPR {
            continue;
        } else {
            break Some(child);
        }
    }
}

// TODO: I'm pretty sure this is not the right way to make
// `!(Boolean())` and others work.
fn skip_grouping_parent(mut parent: SyntaxNode) -> Option<SyntaxNode> {
    loop {
        parent = parent.parent()?;
        if parent.kind() == GROUPING_EXPR {
            continue;
        } else {
            break Some(parent);
        }
    }
}

fn implicitly_casted_node(node: &SyntaxNode) -> Option<SyntaxNode> {
    if matches!(
        node.parent().map(|parent| parent.kind()),
        Some(CONDITION) | Some(FOR_STMT_TEST)
    ) {
        skip_grouping_parent(node.clone())?
            .parent()
            .filter(|node| BOOL_NODE_KINDS.contains(&node.kind()))
    } else {
        skip_grouping_parent(node.clone()).filter(|node| BOOL_NODE_KINDS.contains(&node.kind()))
    }
}

rule_tests! {
    NoExtraBooleanCast::default(),
    err: {
        "if (!!foo) {}",
        "do {} while (!!foo)",
        "while (!!foo) {}",
        "!!foo ? bar : baz",
        "for (; !!foo;) {}",
        "!!!foo",
        "Boolean(!!foo)",
        "new Boolean(!!foo)",
        "if (Boolean(foo)) {}",
        "do {} while (Boolean(foo))",
        "while (Boolean(foo)) {}",
        "Boolean(foo) ? bar : baz",
        "for (; Boolean(foo);) {}",
        "!Boolean(foo)",
        "!Boolean(foo && bar)",
        "!Boolean(foo + bar)",
        "!Boolean(+foo)",
        "!Boolean(foo())",
        "!Boolean(foo = bar)",
        "!Boolean(...foo);",
        "!Boolean(foo, bar());",
        "!Boolean((foo, bar()));",
        "!Boolean();",
        "!(Boolean());",
        "if (!Boolean()) { foo() }",
        "while (!Boolean()) { foo() }",
        "if (Boolean()) { foo() }",
        "while (Boolean()) { foo() }",
        "Boolean(Boolean(foo))",
        "Boolean(!!foo, bar)",
        "x=!!a ? b : c ",
        "void!Boolean()",
        "void! Boolean()",
        "typeof!Boolean()",
        "(!Boolean())",
        "+!Boolean()",
        "void !Boolean()",
        "void(!Boolean())",
        "void/**/!Boolean()",
        "!/**/!!foo",
        "!!/**/!foo",
        "!!!/**/foo",
        "!!!foo/**/",
        "if(!/**/!foo);",
        "(!!/**/foo ? 1 : 2)",
        "!/**/Boolean(foo)",
        "!Boolean/**/(foo)",
        "!Boolean(/**/foo)",
        "!Boolean(foo/**/)",
        "!Boolean(foo)/**/",
        "if(Boolean/**/(foo));",
        "(Boolean(foo/**/) ? 1 : 2)",
        "/**/!Boolean()",
        "!/**/Boolean()",
        "!Boolean/**/()",
        "!Boolean(/**/)",
        "!Boolean()/**/",
        "if(!/**/Boolean());",
        "(!Boolean(/**/) ? 1 : 2)",
        "if(/**/Boolean());",
        "if(Boolean/**/());",
        "if(Boolean(/**/));",
        "if(Boolean()/**/);",
        "(Boolean/**/() ? 1 : 2)",
        "Boolean(!!(a, b))",
        "Boolean(Boolean((a, b)))",
        "Boolean((!!(a, b)))",
        "Boolean((Boolean((a, b))))",
        "Boolean(!(!(a, b)))",
        "Boolean((!(!(a, b))))",
        "Boolean(!!(a = b))",
        "Boolean((!!(a = b)))",
        "Boolean(Boolean(a = b))",
        "Boolean(Boolean((a += b)))",
        "Boolean(!!(a === b))",
        "Boolean(!!((a !== b)))",
        "Boolean(!!a.b)",
        "Boolean(Boolean((a)))",
        "Boolean((!!(a)))",
        "new Boolean(!!(a, b))",
        "new Boolean(Boolean((a, b)))",
        "new Boolean((!!(a, b)))",
        "new Boolean((Boolean((a, b))))",
        "new Boolean(!(!(a, b)))",
        "new Boolean((!(!(a, b))))",
        "new Boolean(!!(a = b))",
        "new Boolean((!!(a = b)))",
        "new Boolean(Boolean(a = b))",
        "new Boolean(Boolean((a += b)))",
        "new Boolean(!!(a === b))",
        "new Boolean(!!((a !== b)))",
        "new Boolean(!!a.b)",
        "new Boolean(Boolean((a)))",
        "new Boolean((!!(a)))",
        "if (!!(a, b));",
        "if (Boolean((a, b)));",
        "if (!(!(a, b)));",
        "if (!!(a = b));",
        "if (Boolean(a = b));",
        "if (!!(a > b));",
        "if (Boolean(a === b));",
        "if (!!f(a));",
        "if (Boolean(f(a)));",
        "if (!!(f(a)));",
        "if ((!!f(a)));",
        "if ((Boolean(f(a))));",
        "if (!!a);",
        "if (Boolean(a));",
        "while (!!(a, b));",
        "while (Boolean((a, b)));",
        "while (!(!(a, b)));",
        "while (!!(a = b));",
        "while (Boolean(a = b));",
        "while (!!(a > b));",
        "while (Boolean(a === b));",
        "while (!!f(a));",
        "while (Boolean(f(a)));",
        "while (!!(f(a)));",
        "while ((!!f(a)));",
        "while ((Boolean(f(a))));",
        "while (!!a);",
        "while (Boolean(a));",
        "do {} while (!!(a, b));",
        "do {} while (Boolean((a, b)));",
        "do {} while (!(!(a, b)));",
        "do {} while (!!(a = b));",
        "do {} while (Boolean(a = b));",
        "do {} while (!!(a > b));",
        "do {} while (!!f(a));",
        "do {} while (Boolean(f(a)));",
        "do {} while (!!(f(a)));",
        "do {} while ((!!f(a)));",
        "do {} while ((Boolean(f(a))));",
        "do {} while (!!a);",
        "do {} while (Boolean(a));",
        "for (; !!(a, b););",
        "for (; Boolean((a, b)););",
        "for (; !(!(a, b)););",
        "for (; !!(a = b););",
        "for (; Boolean(a = b););",
        "for (; !!(a > b););",
        "for (; Boolean(a === b););",
        "for (; !!f(a););",
        "for (; Boolean(f(a)););",
        "for (; !!(f(a)););",
        "for (; (!!f(a)););",
        "for (; (Boolean(f(a))););",
        "for (; !!a;);",
        "for (; Boolean(a););",
        "!!(a, b) ? c : d",
        "(!!(a, b)) ? c : d",
        "Boolean((a, b)) ? c : d",
        "!!(a = b) ? c : d",
        "Boolean(a -= b) ? c : d",
        "(Boolean((a *= b))) ? c : d",
        "!!(a ? b : c) ? d : e",
        "Boolean(a ? b : c) ? d : e",
        "!!(a || b) ? c : d",
        "Boolean(a && b) ? c : d",
        "!!(a === b) ? c : d",
        "Boolean(a < b) ? c : d",
        "!!((a !== b)) ? c : d",
        "Boolean((a >= b)) ? c : d",
        "!!+a ? b : c",
        "!!+(a) ? b : c",
        "Boolean(!a) ? b : c",
        "!!f(a) ? b : c",
        "(!!f(a)) ? b : c",
        "Boolean(a.b) ? c : d",
        "!!a ? b : c",
        "Boolean(a) ? b : c",
        "!!!(a, b)",
        "!Boolean((a, b))",
        "!!!(a = b)",
        "!!(!(a += b))",
        "!(!!(a += b))",
        "!Boolean(a -= b)",
        "!Boolean((a -= b))",
        "!(Boolean(a -= b))",
        "!!!(a || b)",
        "!Boolean(a || b)",
        "!!!(a && b)",
        "!Boolean(a && b)",
        "!!!(a != b)",
        "!!!(a === b)",
        "var x = !Boolean(a > b)",
        "!!!(a - b)",
        "!!!(a ** b)",
        "!Boolean(a ** b)",
        "!Boolean(!a)",
        "!Boolean((!a))",
        "!Boolean(!(a))",
        "!(Boolean(!a))",
        "!!!+a",
        "!!!(+a)",
        "!!(!+a)",
        "!(!!+a)",
        "!Boolean((-a))",
        "!Boolean(-(a))",
        "!!!(--a)",
        "!Boolean(a++)",
        "!!!f(a)",
        "!!!(f(a))",
        "!!!a",
        "!Boolean(a)",
        "!Boolean(!!a)",
        "!Boolean(Boolean(a))",
        "!Boolean(Boolean(!!a))",
        "while (a) { if (!!b) {} }",
        "while (a) { if (Boolean(b)) {} }",
        "if (a) { const b = !!!c; }",
        "if (a) { const b = !Boolean(c); }",
        "for (let a = 0; a < n; a++) { if (!!b) {} }",
        "for (let a = 0; a < n; a++) { if (Boolean(b)) {} }",
        "do { const b = !!!c; } while(a)",
        "do { const b = !Boolean(c); } while(a)"
    },
    ok: {
        "Boolean(bar, !!baz);",
        "var foo = !!bar;",
        "function foo() { return !!bar; }",
        "var foo = bar() ? !!baz : !!bat",
        "for(!!foo;;) {}",
        "for(;; !!foo) {}",
        "var foo = Boolean(bar);",
        "function foo() { return Boolean(bar); }",
        "var foo = bar() ? Boolean(baz) : Boolean(bat)",
        "for(Boolean(foo);;) {}",
        "for(;; Boolean(foo)) {}",
        "if (new Boolean(foo)) {}"
    }
}
