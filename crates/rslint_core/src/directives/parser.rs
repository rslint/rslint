use super::{
    lexer::{Lexer, Token},
    Component, ComponentKind, CstRuleStore, Directive, Instruction,
};
use rslint_errors::Diagnostic;
use rslint_lexer::SyntaxKind;
use rslint_parser::{util::Comment, JsNum, SyntaxNode, SyntaxToken, SyntaxTokenExt};

/// A string that denotes that start of a directive (`rslint-`).
pub const DECLARATOR: &str = "rslint-";

pub type Command = Vec<Instruction>;
pub type Result<T, E = Diagnostic> = std::result::Result<T, E>;

pub struct DirectiveParser {
    /// The root node of a file, `SCRIPT` or `MODULE`.
    root: SyntaxNode,
    file_id: usize,
    commands: Vec<Command>,
}

impl DirectiveParser {
    /// Create a new `DirectivesParser` with a root of a file.
    ///
    /// # Panics
    ///
    /// If the given `root` is not `SCRIPT` or `MODULE`.
    pub fn new(root: SyntaxNode, file_id: usize, commands: Vec<Command>) -> Self {
        assert!(matches!(
            root.kind(),
            SyntaxKind::SCRIPT | SyntaxKind::MODULE
        ));

        Self {
            root,
            file_id,
            commands,
        }
    }

    fn err(&self, msg: &str) -> Diagnostic {
        Diagnostic::error(self.file_id, "directives", msg)
    }

    pub fn top_level_directives(&mut self) -> Result<Vec<Directive>> {
        self.root
            .children_with_tokens()
            .flat_map(|item| item.into_token()?.comment())
            .filter(|comment| comment.content.trim_start().starts_with(DECLARATOR))
            .map(|comment| self.parse_directive(comment))
            .collect::<Result<Vec<_>>>()
    }

    /// Parses a directive, based on all commands inside this `DirectivesParser`.
    fn parse_directive(&mut self, comment: Comment) -> Result<Directive> {
        let text = comment
            .content
            .trim_start()
            .strip_prefix(DECLARATOR)
            .unwrap();

        let decl_offset = comment.content.len() - text.len();
        let offset = usize::from(comment.token.text_range().start()) + decl_offset + 1;

        let mut lexer = Lexer::new(text, self.file_id, offset);

        if matches!(
            lexer.peek(),
            Some(Token {
                kind: SyntaxKind::EOF,
                ..
            })
        ) {
            let range = lexer.next().unwrap().range;
            let d = self
                .err("expected command name, but the comment ends here")
                .primary(range, "");
            return Err(d);
        }

        let cmd_tok = lexer.next().unwrap();
        let cmd_name = lexer.source_of(&cmd_tok);

        let cmd = self
            .commands
            .iter()
            .filter(|cmd| matches!(cmd.first(), Some(Instruction::CommandName(name)) if name.eq_ignore_ascii_case(cmd_name)))
            .next();

        let cmd = match cmd {
            Some(cmd) => cmd.clone(),
            None => {
                // TODO: Suggest name using `find_best_match_for_name`
                let d = self
                    .err(&format!("unknown directive command: `{}`", cmd_name))
                    .primary(cmd_tok.range, "");

                return Err(d);
            }
        };

        let components = self.parse_command(
            &mut lexer,
            Component {
                kind: ComponentKind::CommandName(cmd_name.into()),
                range: cmd_tok.range,
            },
            &cmd,
        )?;
        Ok(Directive {
            comment,
            components,
        })
    }

    fn parse_command(
        &mut self,
        lexer: &mut Lexer<'_>,
        first_component: Component,
        cmd: &Command,
    ) -> Result<Vec<Component>> {
        let mut components = vec![first_component];

        for insn in &cmd[1..] {
            let component = self.parse_instruction(lexer, insn)?;
            components.push(component);
        }

        Ok(components)
    }

    fn parse_instruction(
        &mut self,
        lexer: &mut Lexer<'_>,
        insn: &Instruction,
    ) -> Result<Component> {
        match insn {
            Instruction::CommandName(_) => {
                panic!("command name is only allowed as the first element")
            }
            Instruction::Number => {
                let tok = lexer.expect(SyntaxKind::NUMBER)?;
                let num = lexer.source_of(&tok);
                let num = match rslint_parser::parse_js_num(num.to_string()) {
                    Some(JsNum::Float(val)) => val as u64,
                    Some(JsNum::BigInt(_)) => {
                        let d = self
                            .err("bigints are not supported in directives")
                            .primary(tok.range, "");
                        return Err(d);
                    }
                    _ => {
                        let d = self.err("invalid number").primary(tok.range, "");
                        return Err(d);
                    }
                };
                Ok(Component {
                    kind: ComponentKind::Number(num),
                    range: tok.range,
                })
            }
            Instruction::RuleName => todo!(),
            Instruction::Literal(lit) => {
                let tok = lexer.expect(SyntaxKind::IDENT)?;
                let src = lexer.source_of(&tok);

                if !src.eq_ignore_ascii_case(lit) {
                    let d = self
                        .err(&format!(
                            "expected literal `{}`, but found literal `{}`",
                            lit, src
                        ))
                        .primary(tok.range, "");
                    Err(d)
                } else {
                    Ok(Component {
                        kind: ComponentKind::Literal(lit),
                        range: tok.range,
                    })
                }
            }
            Instruction::Optional(_) => todo!(),
            Instruction::Repetition(_, _) => todo!(),
            Instruction::Either(_, _) => todo!(),
        }
    }
}
