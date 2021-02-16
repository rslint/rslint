use crate::rule_prelude::*;
use ast::*;
use rslint_parser::{Direction, TextRange};
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow confusing newlines in expressions.

    JavaScript has automatic semicolon insertion, where newlines end statements, however,
    expressions can often span across newlines, therefore it can become a bit confusing at times
    and ambiguous. Take the following as an example:

    ```js
    let foo = bar
    /bar/g.test("foo");
    ```

    you would expect this to be a variable declaration and then a regex test, however, it is actually
    a division expression as such: `(bar / bar) / (g.test("foo")).
    This rule is aimed at preventing ambiguous and buggy expressions such like these. It disallows
    ambiguous tagged templates, property accesses, function calls, and division expressions.

    ## Invalid Code Examples

    ```js
    var foo = bar
    (1 || 2).baz();

    var foo = 'bar'
    [1, 2, 3].forEach(addNumber);

    let x = function() {}
    `foo`

    let x = function() {}
    x
    `bar`

    let x = foo
    /regex/g.test(bar)
    ```

    ## Correct Code Examples

    ```js
    var foo = bar;
    (1 || 2).baz();

    var foo = 'bar';
    [1, 2, 3].forEach(addNumber);

    let x = function() {};
    `foo`

    let x = function() {};
    x;
    `bar`

    let x = foo;
    /regex/g.test(bar)
    ```
    */
    #[derive(Default)]
    NoUnexpectedMultiline,
    errors,
    tags(Recommended),
    "no-unexpected-multiline"
}

#[typetag::serde]
impl CstRule for NoUnexpectedMultiline {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        match node.kind() {
            CALL_EXPR => {
                let expr = node.to::<CallExpr>();
                let lhs = expr.callee()?;
                let args = expr.arguments()?;

                if has_linebreak_after(lhs.syntax().siblings_with_tokens(Direction::Next).skip(1))
                    && args.args().count() != 0
                    && expr.opt_chain_token().is_none()
                {
                    let err = ctx
                        .err(self.name(), "ambiguous multiline in function call")
                        .primary(expr.range(), "this is a function call")
                        .secondary(
                            args.range(),
                            "but it could be mistaken for a grouping expression",
                        );

                    ctx.add_err(err);
                }
            }
            TEMPLATE => {
                let template = node.to::<Template>();
                let tag = template.tag()?;

                if has_linebreak_after(tag.syntax().siblings_with_tokens(Direction::Next).skip(1)) {
                    let err = ctx
                        .err(self.name(), "ambiguous multiline in tagged template")
                        .primary(template.range(), "this is a tagged template")
                        .secondary(
                            template.template_range()?,
                            "but it could be mistaken for an expression plus an untagged template",
                        );

                    ctx.add_err(err);
                }
            }
            BRACKET_EXPR => {
                let expr = node.to::<BracketExpr>();
                let start = expr.l_brack_token()?.text_range().start();

                if has_linebreak_after(
                    expr.object()?
                        .syntax()
                        .siblings_with_tokens(Direction::Next)
                        .skip(1),
                ) && expr.opt_chain_token().is_none()
                {
                    let err = ctx
                        .err(self.name(), "ambiguous multiline in property access")
                        .primary(expr.range(), "this is a property access")
                        .secondary(
                            TextRange::new(start, expr.range().end()),
                            "but it could be mistaken for an array literal",
                        );

                    ctx.add_err(err);
                }
            }
            BIN_EXPR => {
                const FLAGS: [char; 6] = ['g', 'i', 'm', 's', 'u', 'y'];

                let expr = node.to::<BinExpr>();
                let op = expr.op()?;
                let parent = node
                    .parent()?
                    .try_to::<BinExpr>()
                    .filter(|x| x.op() == Some(op!(/)))?;
                let op_range = parent.op_token().unwrap().text_range();

                let flags = parent.syntax().lossy_tokens().into_iter().find(|tok| {
                    tok.text_range().start() == op_range.end() && tok.kind() == IDENT
                })?;
                if !flags.text().chars().all(|x| FLAGS.contains(&x)) {
                    return None;
                }

                if op == op!(/)
                    && has_linebreak_after(
                        expr.lhs()?
                            .syntax()
                            .siblings_with_tokens(Direction::Next)
                            .skip(1),
                    )
                {
                    let range = TextRange::new(
                        expr.op_token().unwrap().text_range().start(),
                        parent.rhs().unwrap().range().end(),
                    );

                    let err = ctx
                        .err(self.name(), "ambiguous multiline in divison expression")
                        .primary(parent.range(), "this is a division expression")
                        .secondary(range, "but it could be mistaken for a RegEx");

                    ctx.add_err(err);
                }
            }
            _ => {}
        }

        None
    }
}

fn has_linebreak_after(siblings: impl Iterator<Item = SyntaxElement>) -> bool {
    let mut tokens = siblings.scan((), |_, elem| {
        elem.into_token().filter(|x| x.kind().is_trivia())
    });
    tokens.any(|tok| parseutil::contains_js_linebreak(tok.text().as_str()))
}

rule_tests! {
    NoUnexpectedMultiline::default(),
    err: {
        "var a = b\n(x || y).doSomething()",
        "var a = (a || b)\n(x || y).doSomething()",
        "var a = (a || b)\n(x).doSomething()",
        "var a = b\n[a, b, c].forEach(doSomething)",
        "var a = b\n    (x || y).doSomething()",
        "var a = b\n  [a, b, c].forEach(doSomething)",
        "let x = function() {}\n `hello`",
        "let x = function() {}\nx\n`hello`",
        "x\n.y\nz\n`Invalid Test Case`",
        "
            foo
            / bar /gym
        ",
        "
            foo
            / bar /g
        ",
        "
            foo
            / bar /g.test(baz)
        "
    },
    ok: {
        "(x || y).aFunction()",
        "[a, b, c].forEach(doSomething)",
        "var a = b;\n(x || y).doSomething()",
        "var a = b\n;(x || y).doSomething()",
        "var a = b\nvoid (x || y).doSomething()",
        "var a = b;\n[1, 2, 3].forEach(console.log)",
        "var a = b\nvoid [1, 2, 3].forEach(console.log)",
        "\"abc\\\n(123)\"",
        "var a = (\n(123)\n)",
        "f(\n(x)\n)",
        "(\nfunction () {}\n)[1]",
        "let x = function() {};\n   `hello`",
        "let x = function() {}\nx `hello`",
        "String.raw `Hi\n${2+3}!`;",
        "x\n.y\nz `Valid Test Case`",
        "f(x\n)`Valid Test Case`",
        "x.\ny `Valid Test Case`",
        "(x\n)`Valid Test Case`",
        "
            foo
            / bar /2
        ",
        "
        foo
        / bar / mgy
        ",
        "
        foo
        / bar /
        gym
        ",
        "
        foo
        / bar
        / ygm
        ",
        "
        foo
        / bar /GYM
        ",
        "
            foo
            / bar / baz
        ",
        "foo /bar/g",
        "
            foo
            /denominator/
            2
        ",
        "
            foo
            / /abc/
        ",
        "
            5 / (5
            / 5)
        ",
        "var a = b\n  ?.(x || y).doSomething()",
        "var a = b\n  ?.[a, b, c].forEach(doSomething)",
        "var a = b?.\n  (x || y).doSomething()",
        "var a = b?.\n  [a, b, c].forEach(doSomething)",
    }
}
