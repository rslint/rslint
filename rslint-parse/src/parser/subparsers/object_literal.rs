// use crate::parser::Parser;
// use crate::diagnostic::ParserDiagnostic;
// use crate::parser::cst::expr::*;
// use crate::lexer::token::*;
// use crate::span::Span;
// use crate::parser::error::ParseDiagnosticType::*;

// static obj_acceptable: [TokenType; 5] = [TokenType::Identifier, TokenType::LiteralNumber, TokenType::LiteralString, TokenType::Comma, TokenType::BraceClose];

// impl<'a> Parser<'a> {
//   // Expects current token to be the opening brace.
//   pub fn parse_object_literal(&mut self) -> Result<Object, ParserDiagnostic<'a>> {
//     debug_assert_eq!(self.cur_tok.token_type, TokenType::BraceOpen);

//     let start = self.cur_tok.lexeme.start;
//     let mut props: Vec<ObjProp> = vec![];
//     let leading_ws = self.get_whitespace()?;

//     match self.advance_lexer()?.map(|x| x.token_type) {
//       Some(TokenType::Identifier) | Some(TokenType::LiteralNumber) | Some(TokenType::LiteralString) => {},

//       Some(TokenType::BraceClose) => {
//         return Ok(Object {
//           span: Span::new(start, self.cur_tok.lexeme.end),
//           props,
//           has_trailing_comma: false,
//         });
//       },

//       Some(TokenType::Comma) => {
//         // We can recover by just pretending the comma wasnt there
//         let err = self.error(InvalidCommaInsideObject, "Invalid comma without preceding property inside object literal")
//           .primary(self.cur_tok.lexeme.to_owned(), "Expected a property before this comma");

//         self.errors.push(err);
//       },

//       None => {
//         // We will recover by just assuming the object is an empty object
//         let err = self.error(UnterminatedObjectLiteral, "Unexpected end of file resulting in unterminated object literal")
//           .secondary(start..start + 1, "Object literal starts here")
//           .primary(start..self.cur_tok.lexeme.end, "File ends here");

//         self.errors.push(err);
//         return Ok(Object {
//           span: Span::new(start, self.cur_tok.lexeme.end),
//           props,
//           has_trailing_comma: false,
//         });
//       },

//       _ => {
//         // Try to recover by throwing out tokens until a valid one is found, this is more dangerous.
//         // Throwing out too many tokens can yield random errors
//         self.discard_recover(|x| !obj_acceptable.contains(x))?;
//       }
//     }

//     loop {
//       match self.cur_tok.token_type {
//         TokenType::Identifier | TokenType::LiteralNumber | TokenType::LiteralString => {
//           let prop = self.parse_object_property(leading_ws.to_owned())?;
//           match self.cur_tok.token_type {
//             TokenType::Comma => {
//               // {"a": "b",}
//               if self.peek_lexer()?.map(|x| x.token_type) == Some(TokenType::BraceClose) {
//                 self.advance_lexer()?;
//                 return Ok(Object {
//                   span: Span::new(start, self.cur_tok.lexeme.end),
//                   props,
//                   has_trailing_comma: true
//                 });
//               }
//             },

//           }
//         }
//       }
//     }
//   }

//   // Expects current token to be either a string, number, or identifier
//   fn parse_object_property(&mut self, leading_whitespace: Span) -> Result<ObjProp, ParserDiagnostic<'a>> {
//     let key_tok = self.cur_tok.to_owned();
//     // { "a" : "b"}
//     //  ^   ^
//     let whitespace = ExprWhitespace {
//       before: leading_whitespace.to_owned(),
//       after: self.get_whitespace()?,
//     };
//     let lexeme = self.cur_tok.lexeme.to_owned();
//     let key = match key_tok.token_type {
//       TokenType::LiteralString => ObjPropKey::LiteralString(lexeme, whitespace.to_owned()),
//       TokenType::LiteralNumber => ObjPropKey::LiteralNumber(lexeme, whitespace.to_owned()),
//       TokenType::Identifier => ObjPropKey::Identifier(lexeme, whitespace.to_owned()),
//       _ => unreachable!(),
//     };

//     if self.cur_tok.token_type == TokenType::Colon {
//       let before_expr_ws = self.get_whitespace()?;
//       // TODO: change this to parse an expression
//       if self.advance_lexer()?.map(|x| x.token_type) != Some(TokenType::Identifier) {
//         unimplemented!();
//       }
//       let ident_lexeme = self.cur_tok.lexeme.to_owned();
//       let after_expr_ws = self.get_whitespace()?;
//       let whitespace = ExprWhitespace {
//         before: before_expr_ws,
//         after: after_expr_ws
//       };
//       let val = ObjPropVal::Initialized(ident_lexeme.to_owned(), Expr::Identifier(ident_lexeme, whitespace));
//       let prop = ObjProp {
//         span: Span::new(leading_whitespace.start, self.cur_tok.lexeme.start),
//         key,
//         val,
//         whitespace: None,
//       };
//       Ok(prop)
//     } else {
//       match self.cur_tok.token_type {
//         // TODO: change this to expression
//         TokenType::Identifier => {
//           // We can recover from this by just assuming a colon was there
//           // This "inserts" the colon directly after the key
//           self.errors.push(
//             self.error(MissingColonAfterKey, "Missing colon after object key")
//               .primary(key_tok.lexeme.range, "Expected a colon after this key")
//           );
//           let mut prop_whitespace = whitespace.to_owned();
//           // The value's leading whitespace is unaffected by recovery semicolon insertion
//           let ident_whitespace = ExprWhitespace {
//             before: prop_whitespace.after,
//             after: self.get_whitespace()?
//           };
//           let cur_lexeme = self.cur_tok.lexeme.to_owned();
//           let val = ObjPropVal::Initialized(cur_lexeme.to_owned(), Expr::Identifier(cur_lexeme, ident_whitespace));

//           // Even if the whitespace was something like {"a"  "b"},
//           // We will define the key's trailing whitespace as being zero since the semicolon is "inserted" there
//           prop_whitespace.after = Span::new(key_tok.lexeme.end, key_tok.lexeme.end);
//           let prop = ObjProp {
//             span: Span::new(prop_whitespace.before.start, prop_whitespace.after.end),
//             key,
//             val,
//             whitespace: None
//           };
//           Ok(prop)
//         },
//         _ => panic!()
//       }
//     }
//   }
// }
