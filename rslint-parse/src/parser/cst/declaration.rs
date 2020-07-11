use super::expr::*;
use super::stmt::*;
use crate::span::Span;
use crate::parser::Parser;
use crate::parser::error::ParseDiagnosticType::{InvalidTrailingComma, UnexpectedToken};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Declaration {
    Function(FunctionDecl),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionDecl {
    pub span: Span,
    pub function_whitespace: LiteralWhitespace,
    /// If the name is `None` it means the declaration is an expression
    pub name: Option<LiteralExpr>,
    pub parameters: Parameters,
    pub open_brace_whitespace: LiteralWhitespace,
    pub close_brace_whitespace: LiteralWhitespace,
    pub body: Vec<StmtListItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameters {
    pub span: Span,
    pub parameters: Vec<LiteralExpr>,
    pub comma_whitespaces: Vec<LiteralWhitespace>,
    pub open_paren_whitespace: LiteralWhitespace,
    pub close_paren_whitespace: LiteralWhitespace,
}

impl Arguments {
    /// Convert arguments to function parameters, this allows us to just reuse the call expression code
    pub fn to_parameters(&self, parser: &mut Parser) -> Parameters {
        let mut parameters = Vec::with_capacity(self.arguments.len());
        let mut comma_whitespaces = Vec::with_capacity(self.comma_whitespaces.len());

        // filter out any non identifier args
        for (idx, arg) in self.arguments.iter().enumerate() {
            if let Expr::Identifier(data) = arg {
                parameters.push(data.to_owned());
                if idx != self.arguments.len() - 1 {
                    comma_whitespaces.push(self.comma_whitespaces[idx].to_owned());
                }
            } else {
                let err = parser.error(UnexpectedToken, "Expected an identifier in function declaration, but found an expression")
                    .primary(arg.span().to_owned(), "Only identifiers are allowed in function declarations");

                parser.errors.push(err);
            }
        };

        // Trailing comma isnt allowed
        if self.arguments.len() < self.comma_whitespaces.len() {
            let err = parser.error(InvalidTrailingComma, "Function declaration parameters cannot contain a trailing comma")
                .primary(Span::from(self.comma_whitespaces.last().unwrap().before.end), "Remove this comma");

            parser.errors.push(err);
        }

        Parameters {
            span: self.span,
            parameters,
            comma_whitespaces,
            open_paren_whitespace: self.open_paren_whitespace.to_owned(),
            close_paren_whitespace: self.close_paren_whitespace.to_owned(),
        }
    }
}

impl Parameters {
    /// Verify no declared parameters have the same name or the name arguments or eval if the parser is in strict mode
    pub fn verify_strict_mode<'p>(&self, parser: &'p mut Parser, fn_name: Option<&str>) {
        use crate::parser::error::ParseDiagnosticType::{DisallowedIdentifier, DuplicateFunctionParameters};
        use std::collections::HashMap;

        let mut map: HashMap<&str, Span> = HashMap::new();
        for ident in &self.parameters {
            if ["eval", "arguments"].contains(&ident.span.content(parser.source)) {
                let err = parser.error(DisallowedIdentifier, "`eval` and `arguments` cannot be used as parameters to a function in strict mode")
                    .primary(ident.span, "This is an invalid identifier since the function is in strict mode");

                parser.errors.push(err);
                continue;
            }
            if map.contains_key(&ident.span.content(parser.source)) {
                let decl = ident.span.content(parser.source);
                let function_name = if fn_name.is_some() {
                    format!("in function `{}`", fn_name.unwrap())
                } else {
                    "".to_string()
                };

                let err = parser.error(DuplicateFunctionParameters, &format!("Parameter `{}` {} cannot be used multiple times as the function is in strict mode", decl, &function_name))
                    .secondary(map.get(ident.span.content(parser.source)).unwrap().to_owned(), &format!("`{}` is first declared here", decl))
                    .primary(ident.span, &format!("`{}` is redeclared here", decl));

                parser.errors.push(err);
            } else {
                map.insert(ident.span.content(parser.source), ident.span.to_owned());
            }
        }
    }
}