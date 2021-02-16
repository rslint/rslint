use crate::rule_prelude::*;
use ast::{CallExpr, DotExpr};

declare_lint! {
    /**
    Disallow direct use of `Object.prototype` builtins directly.

    ES 5.1 added `Object.create` which allows creation of object with a custom prototype. This
    pattern is frequently used for objects used as Maps. However this pattern can lead to errors
    if something else relies on prototype properties/methods.

    Moreover, the methods could be shadowed, this can lead to random bugs and denial of service
    vulnerabilities. For example, calling `hasOwnProperty` directly on parsed json could lead to vulnerabilities.
    Instead, you should use get the method directly from the object using `Object.prototype.prop.call(item, args)`.

    ## Invalid Code Examples

    ```js
    var bar = foo.hasOwnProperty("bar");

    var bar = foo.isPrototypeOf(bar);

    var bar = foo.propertyIsEnumerable("bar");
    ```

    ## Correct Code Examples

    ```js
    var bar = Object.prototype.hasOwnProperty.call(foo, "bar");

    var bar = Object.prototype.isPrototypeOf.call(foo, bar);

    var bar = Object.propertyIsEnumerable.call(foo, "bar");
    ```
    */
    #[derive(Default)]
    NoPrototypeBuiltins,
    errors,
    tags(Recommended),
    "no-prototype-builtins"
}

const CHECKED_PROPS: [&str; 3] = ["hasOwnProperty", "isPrototypeOf", "propertyIsEnumberable"];

#[typetag::serde]
impl CstRule for NoPrototypeBuiltins {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let expr = node.try_to::<CallExpr>()?;
        let lhs = expr.callee()?.syntax().try_to::<DotExpr>()?;
        let prop = lhs.prop()?;
        let object = lhs.object()?;

        if CHECKED_PROPS.contains(&prop.text().as_str()) {
            let mut err = ctx
                .err(
                    self.name(),
                    format!(
                        "do not access the object property `{}` directly from `{}`",
                        prop.text(),
                        object.text()
                    ),
                )
                .primary(expr.range(), "");

            err = suggestion(prop.text(), object.text(), expr, err);
            ctx.add_err(err);
        }
        None
    }
}

fn suggestion(prop: String, object: String, expr: CallExpr, err: Diagnostic) -> Diagnostic {
    let arg = if let Some(arg) = expr.arguments().and_then(|args| args.args().next()) {
        format!(", {}", arg.text())
    } else {
        "".to_string()
    };

    let start = format!("Object.prototype.{}.call", prop);
    let suggestion_expr = format!("{}({}{})", start, object, arg);
    err.suggestion_with_labels(
        expr.syntax(),
        "get the function from the prototype of `Object` and call it",
        suggestion_expr,
        Applicability::Always,
        vec![0..start.len()],
    )
    .footer_note(
        "the method may be shadowed and cause random bugs and denial of service vulnerabilities",
    )
}

rule_tests! {
    NoPrototypeBuiltins::default(),
    err: {
        "foo.hasOwnProperty(\"bar\");",
        "foo.isPrototypeOf(\"bar\");",
        "foo.propertyIsEnumberable(\"bar\");",
        "foo.bar.baz.hasOwnProperty(\"bar\");"
    },
    ok: {
        "Object.prototype.hasOwnProperty.call(foo, 'bar');",
        "Object.prototype.isPrototypeOf.call(foo, 'bar');",
        "Object.prototype.propertyIsEnumberable.call(foo, 'bar');",
        "Object.prototype.hasOwnProperty.apply(foo, ['bar']);",
        "Object.prototype.isPrototypeOf.apply(foo, ['bar']);",
        "Object.prototype.propertyIsEnumberable.apply(foo, ['bar']);",
        "hasOwnProperty(foo, 'bar');",
        "isPrototypeOf(foo, 'bar');",
        "propertyIsEnumberable(foo, 'bar');",
        "({}.hasOwnProperty.call(foo, 'bar'));",
        "({}.isPrototypeOf.call(foo, 'bar'));",
        "({}.propertyIsEnumberable.call(foo, 'bar'));",
        "({}.hasOwnProperty.apply(foo, ['bar']));",
        "({}.isPrototypeOf.apply(foo, ['bar']));",
        "({}.propertyIsEnumberable.apply(foo, ['bar']));"
    }
}
