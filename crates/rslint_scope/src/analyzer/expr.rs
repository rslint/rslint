use crate::{datalog::DatalogBuilder, AnalyzerInner, Visit};
use ast::{
    ArrayElement, ClassElement as DatalogClassElement, ExprId, FuncParam,
    Pattern as DatalogPattern, PropertyKey, PropertyVal,
};
use ddlog_std::{tuple2, Either};
use internment::Intern;
use rslint_parser::{
    ast::{
        ArgList, ArrayExpr, ArrowExpr, ArrowExprParams, AssignExpr, AstChildren, AwaitExpr,
        BinExpr, BracketExpr, CallExpr, ClassElement, ClassExpr, CondExpr, DotExpr, Expr,
        ExprOrBlock, ExprOrSpread, FnExpr, GroupingExpr, ImportCall, ImportMeta, Literal,
        LiteralKind, NameRef, NewExpr, NewTarget, ObjectExpr, ObjectProp, ParameterList,
        PatternOrExpr, PropName, SequenceExpr, SuperCall, Template, TemplateElement, ThisExpr,
        UnaryExpr, YieldExpr,
    },
    AstNode, SyntaxNodeExt,
};
use std::iter;
use types::IMPLICIT_ARGUMENTS;

impl<'ddlog> Visit<'ddlog, Expr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, expr: Expr) -> Self::Output {
        match expr {
            Expr::Literal(literal) => self.visit(scope, literal),
            Expr::NameRef(name) => self.visit(scope, name),
            Expr::ArrowExpr(arrow) => self.visit(scope, arrow),
            Expr::Template(template) => self.visit(scope, template),
            Expr::ThisExpr(this) => self.visit(scope, this),
            Expr::ArrayExpr(array) => self.visit(scope, array),
            Expr::ObjectExpr(object) => self.visit(scope, object),
            Expr::GroupingExpr(grouping) => self.visit(scope, grouping),
            Expr::BracketExpr(bracket) => self.visit(scope, bracket),
            Expr::DotExpr(dot) => self.visit(scope, dot),
            Expr::NewExpr(new) => self.visit(scope, new),
            Expr::CallExpr(call) => self.visit(scope, call),
            Expr::UnaryExpr(unary) => self.visit(scope, unary),
            Expr::BinExpr(bin) => self.visit(scope, bin),
            Expr::CondExpr(cond) => self.visit(scope, cond),
            Expr::AssignExpr(assign) => self.visit(scope, assign),
            Expr::SequenceExpr(sequence) => self.visit(scope, sequence),
            Expr::FnExpr(fn_expr) => self.visit(scope, fn_expr),
            Expr::ClassExpr(class) => self.visit(scope, class),
            Expr::NewTarget(target) => self.visit(scope, target),
            Expr::ImportMeta(import) => self.visit(scope, import),
            Expr::SuperCall(super_call) => self.visit(scope, super_call),
            Expr::ImportCall(import) => self.visit(scope, import),
            Expr::YieldExpr(yield_expr) => self.visit(scope, yield_expr),
            Expr::AwaitExpr(await_expr) => self.visit(scope, await_expr),
        }
    }
}

impl<'ddlog> Visit<'ddlog, NameRef> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, name: NameRef) -> Self::Output {
        scope.name_ref(Intern::new(name.to_string()), name.syntax().trimmed_range())
    }
}

impl<'ddlog> Visit<'ddlog, Literal> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, literal: Literal) -> Self::Output {
        let span = literal.syntax().trimmed_range();

        match literal.kind() {
            LiteralKind::Number(number) => scope.number(number, span),
            LiteralKind::BigInt(bigint) => scope.bigint(bigint, span),
            LiteralKind::String => scope.string(
                Intern::new(literal.inner_string_text().unwrap().to_string()),
                span,
            ),
            LiteralKind::Null => scope.null(span),
            LiteralKind::Bool(boolean) => scope.boolean(boolean, span),
            LiteralKind::Regex => scope.regex(span),
        }
    }
}

impl<'ddlog> Visit<'ddlog, YieldExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, yield_expr: YieldExpr) -> Self::Output {
        let expr = self.visit(scope, yield_expr.value());
        scope.yield_expr(expr, yield_expr.range())
    }
}

impl<'ddlog> Visit<'ddlog, AwaitExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, await_expr: AwaitExpr) -> Self::Output {
        let expr = self.visit(scope, await_expr.expr());
        scope.await_expr(expr, await_expr.range())
    }
}

impl<'ddlog> Visit<'ddlog, ArrowExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, arrow: ArrowExpr) -> Self::Output {
        let body = arrow.body().map(|body| {
            let scope = scope.scope();
            let body = match body {
                ExprOrBlock::Expr(expr) => Either::Left {
                    l: self.visit(&scope, expr),
                },
                ExprOrBlock::Block(block) => Either::Right {
                    r: self.visit(&scope, block),
                },
            };

            tuple2(body, scope.scope_id())
        });

        let params = arrow
            .params()
            .map(|params| match params {
                ArrowExprParams::Name(name) => vec![Intern::new(DatalogPattern::SinglePattern {
                    name: Some(self.visit(scope, name)).into(),
                })],

                ArrowExprParams::ParameterList(params) => params
                    .parameters()
                    .map(|pattern| self.visit(scope, pattern))
                    .collect(),
            })
            .unwrap_or_default();

        scope.arrow(body, params, arrow.range())
    }
}

impl<'ddlog> Visit<'ddlog, UnaryExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, unary: UnaryExpr) -> Self::Output {
        let op = unary.op().map(Into::into);
        let expr = self.visit(scope, unary.expr());

        scope.unary(op, expr, unary.range())
    }
}

impl<'ddlog> Visit<'ddlog, BinExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, bin: BinExpr) -> Self::Output {
        let op = bin.op().map(Into::into);
        let lhs = self.visit(scope, bin.lhs());
        let rhs = self.visit(scope, bin.rhs());

        scope.bin(op, lhs, rhs, bin.range())
    }
}

impl<'ddlog> Visit<'ddlog, CondExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, cond: CondExpr) -> Self::Output {
        let test = self.visit(scope, cond.test());
        let true_val = self.visit(scope, cond.cons());
        let false_val = self.visit(scope, cond.alt());

        scope.ternary(test, true_val, false_val, cond.range())
    }
}

impl<'ddlog> Visit<'ddlog, Template> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, template: Template) -> Self::Output {
        let tag = self.visit(scope, template.tag());
        let elements = self.visit(scope, template.elements());

        scope.template(tag, elements, template.range())
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<TemplateElement>> for AnalyzerInner {
    type Output = Vec<ExprId>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        elements: AstChildren<TemplateElement>,
    ) -> Self::Output {
        elements
            .filter_map(|elem| self.visit(scope, elem.expr()))
            .collect()
    }
}

impl<'ddlog> Visit<'ddlog, ThisExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, this: ThisExpr) -> Self::Output {
        scope.this(this.range())
    }
}

impl<'ddlog> Visit<'ddlog, ArrayExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, array: ArrayExpr) -> Self::Output {
        let elements = self.visit(scope, array.elements());
        scope.array(elements, array.range())
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<ExprOrSpread>> for AnalyzerInner {
    type Output = Vec<ArrayElement>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        elements: AstChildren<ExprOrSpread>,
    ) -> Self::Output {
        elements
            .map(|elem| match elem {
                ExprOrSpread::Expr(expr) => ArrayElement::ArrExpr {
                    expr: self.visit(scope, expr),
                },
                ExprOrSpread::Spread(spread) => ArrayElement::ArrSpread {
                    spread: self.visit(scope, spread.element()).into(),
                },
            })
            .collect()
    }
}

impl<'ddlog> Visit<'ddlog, ObjectExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, object: ObjectExpr) -> Self::Output {
        let properties = self.visit(scope, object.props());
        scope.object(properties, object.range())
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<ObjectProp>> for AnalyzerInner {
    type Output = Vec<(Option<PropertyKey>, PropertyVal)>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        properties: AstChildren<ObjectProp>,
    ) -> Self::Output {
        // TODO: Break into separate visitors?
        properties
            .map(|prop| match prop {
                ObjectProp::LiteralProp(literal) => {
                    let key = self.visit(scope, literal.key());
                    let lit = self.visit(scope, literal.value()).into();

                    (key, PropertyVal::PropLit { lit })
                }

                ObjectProp::Getter(getter) => {
                    let key = self.visit(scope, getter.key());
                    let body = self.visit(scope, getter.body()).into();

                    (key, PropertyVal::PropGetter { body })
                }

                ObjectProp::Setter(setter) => {
                    let key = self.visit(scope, setter.key());
                    let params = self
                        .visit(scope, setter.parameters())
                        .map(|params| {
                            params
                                .into_iter()
                                .map(FuncParam::explicit)
                                .chain(iter::once(FuncParam::implicit(IMPLICIT_ARGUMENTS.clone())))
                                .collect::<Vec<_>>()
                                .into()
                        })
                        .into();

                    let body_scope = scope.scope();
                    let body = self.visit(&body_scope, setter.body()).into();

                    (key, PropertyVal::PropSetter { params, body })
                }

                ObjectProp::SpreadProp(spread) => {
                    let value = self.visit(scope, spread.value()).into();

                    (None, PropertyVal::PropSpread { value })
                }

                ObjectProp::InitializedProp(init) => {
                    let key = init.key().map(|ident| PropertyKey::IdentKey {
                        ident: self.visit(scope, ident),
                    });
                    let value = self.visit(scope, init.value()).into();

                    (key, PropertyVal::PropSpread { value })
                }

                ObjectProp::IdentProp(ident) => {
                    let key = ident.name().map(|ident| PropertyKey::IdentKey {
                        ident: self.visit(scope, ident),
                    });

                    (key, PropertyVal::PropIdent)
                }

                ObjectProp::Method(method) => {
                    let key = self.visit(scope, method.name());
                    let params = self
                        .visit(scope, method.parameters())
                        .map(|params| {
                            params
                                .into_iter()
                                .map(FuncParam::explicit)
                                .chain(iter::once(FuncParam::implicit(IMPLICIT_ARGUMENTS.clone())))
                                .collect::<Vec<_>>()
                                .into()
                        })
                        .into();
                    let body = self.visit(scope, method.body()).into();

                    (key, PropertyVal::PropMethod { params, body })
                }
            })
            .collect()
    }
}

impl<'ddlog> Visit<'ddlog, ParameterList> for AnalyzerInner {
    type Output = Vec<Intern<DatalogPattern>>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, params: ParameterList) -> Self::Output {
        params
            .parameters()
            .map(|param| self.visit(scope, param))
            .collect()
    }
}

impl<'ddlog> Visit<'ddlog, PropName> for AnalyzerInner {
    type Output = PropertyKey;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, prop: PropName) -> Self::Output {
        match prop {
            PropName::Computed(computed) => PropertyKey::ComputedKey {
                prop: self.visit(scope, computed.prop()).into(),
            },
            PropName::Literal(literal) => PropertyKey::LiteralKey {
                lit: self.visit(scope, literal),
            },
            PropName::Ident(ident) => PropertyKey::IdentKey {
                ident: self.visit(scope, ident),
            },
        }
    }
}

impl<'ddlog> Visit<'ddlog, GroupingExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, grouping: GroupingExpr) -> Self::Output {
        let inner = self.visit(scope, grouping.inner());
        scope.grouping(inner, grouping.range())
    }
}

impl<'ddlog> Visit<'ddlog, BracketExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, bracket: BracketExpr) -> Self::Output {
        let object = self.visit(scope, bracket.object());
        let property = self.visit(scope, bracket.prop());

        scope.bracket(object, property, bracket.range())
    }
}

impl<'ddlog> Visit<'ddlog, DotExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, dot: DotExpr) -> Self::Output {
        let object = self.visit(scope, dot.object());
        let property = self.visit(scope, dot.prop());

        scope.dot(object, property, dot.range())
    }
}

impl<'ddlog> Visit<'ddlog, NewExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, new: NewExpr) -> Self::Output {
        let object = self.visit(scope, new.object());
        let args = self.visit(scope, new.arguments());

        scope.new(object, args, new.range())
    }
}

impl<'ddlog> Visit<'ddlog, ArgList> for AnalyzerInner {
    type Output = Vec<ExprId>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, args: ArgList) -> Self::Output {
        args.args().map(|arg| self.visit(scope, arg)).collect()
    }
}

impl<'ddlog> Visit<'ddlog, CallExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, call: CallExpr) -> Self::Output {
        let callee = self.visit(scope, call.callee());
        let args = self.visit(scope, call.arguments());

        scope.call(callee, args, call.range())
    }
}

impl<'ddlog> Visit<'ddlog, AssignExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, assign: AssignExpr) -> Self::Output {
        let lhs = self.visit(scope, assign.lhs());
        let rhs = self.visit(scope, assign.rhs());
        let op = assign.op().map(Into::into);

        scope.assign(lhs, rhs, op, assign.range())
    }
}

impl<'ddlog> Visit<'ddlog, PatternOrExpr> for AnalyzerInner {
    type Output = Either<Intern<DatalogPattern>, ExprId>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, either: PatternOrExpr) -> Self::Output {
        match either {
            PatternOrExpr::Pattern(pattern) => Either::Left {
                l: self.visit(scope, pattern),
            },
            PatternOrExpr::Expr(expr) => Either::Right {
                r: self.visit(scope, expr),
            },
        }
    }
}

impl<'ddlog> Visit<'ddlog, SequenceExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, sequence: SequenceExpr) -> Self::Output {
        let exprs = self.visit(scope, sequence.exprs());
        scope.sequence(exprs, sequence.range())
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<Expr>> for AnalyzerInner {
    type Output = Vec<ExprId>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, exprs: AstChildren<Expr>) -> Self::Output {
        exprs.map(|expr| self.visit(scope, expr)).collect()
    }
}

impl<'ddlog> Visit<'ddlog, NewTarget> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, target: NewTarget) -> Self::Output {
        scope.new_target(target.range())
    }
}

impl<'ddlog> Visit<'ddlog, ImportMeta> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, import: ImportMeta) -> Self::Output {
        scope.import_meta(import.range())
    }
}

impl<'ddlog> Visit<'ddlog, FnExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, fn_expr: FnExpr) -> Self::Output {
        let name = self.visit(scope, fn_expr.name());
        let params = self.visit(scope, fn_expr.parameters()).unwrap_or_default();
        let body = self.visit(scope, fn_expr.body());

        scope.fn_expr(name, params, body, fn_expr.range())
    }
}

impl<'ddlog> Visit<'ddlog, SuperCall> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, super_call: SuperCall) -> Self::Output {
        let args = self.visit(scope, super_call.arguments());
        scope.super_call(args, super_call.range())
    }
}

impl<'ddlog> Visit<'ddlog, ImportCall> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, import: ImportCall) -> Self::Output {
        let arg = self.visit(scope, import.argument());
        scope.import_call(arg, import.range())
    }
}

impl<'ddlog> Visit<'ddlog, ClassExpr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, class: ClassExpr) -> Self::Output {
        let elements = self.visit(scope, class.body().map(|body| body.elements()));
        scope.class_expr(elements, class.range())
    }
}

impl<'ddlog> Visit<'ddlog, ClassElement> for AnalyzerInner {
    type Output = Intern<DatalogClassElement>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, elem: ClassElement) -> Self::Output {
        Intern::new(match elem {
            ClassElement::EmptyStmt(_empty) => DatalogClassElement::ClassEmptyElem,

            ClassElement::Method(method) => {
                let name = self.visit(scope, method.name()).into();
                let params = self
                    .visit(scope, method.parameters())
                    .map(|params| {
                        params
                            .into_iter()
                            .map(FuncParam::explicit)
                            .chain(iter::once(FuncParam::implicit(IMPLICIT_ARGUMENTS.clone())))
                            .collect::<Vec<_>>()
                            .into()
                    })
                    .into();
                let body = self.visit(scope, method.body()).into();

                DatalogClassElement::ClassMethod { name, params, body }
            }

            ClassElement::StaticMethod(static_method) => {
                let method = static_method.method();
                let method = method.as_ref();

                let name = self.visit(scope, method.and_then(|m| m.name())).into();
                let params = self
                    .visit(scope, method.and_then(|m| m.parameters()))
                    .map(|params| {
                        params
                            .into_iter()
                            .map(FuncParam::explicit)
                            .chain(iter::once(FuncParam::implicit(IMPLICIT_ARGUMENTS.clone())))
                            .collect::<Vec<_>>()
                            .into()
                    })
                    .into();
                let body = self.visit(scope, method.and_then(|m| m.body())).into();

                DatalogClassElement::ClassStaticMethod { name, params, body }
            }
        })
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<ClassElement>> for AnalyzerInner {
    type Output = Vec<Intern<DatalogClassElement>>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        elements: AstChildren<ClassElement>,
    ) -> Self::Output {
        elements.map(|element| self.visit(scope, element)).collect()
    }
}
