use crate::{get_rule_suggestion, CstRuleStore, File};

use super::{
    commands::Command,
    get_command_descriptors,
    lexer::{format_kind, Lexer, Token},
    CommandDescriptor, Component, ComponentKind, Directive, Instruction,
};
use rslint_errors::{file::line_starts, Diagnostic};
use rslint_lexer::{SyntaxKind, T};
use rslint_parser::{
    ast::ModuleItem, util::Comment, JsNum, SyntaxNode, SyntaxNodeExt, SyntaxTokenExt, TextRange,
};
use std::ops::Range;

/// A string that denotes that start of a directive (`rslint-`).
pub const DECLARATOR: &str = "rslint-";

pub type Result<T, E = DirectiveError> = std::result::Result<T, E>;

/// The result of a parsed directive.
#[derive(Default)]
pub struct DirectiveResult {
    pub directives: Vec<Directive>,
    pub diagnostics: Vec<DirectiveError>,
}

impl DirectiveResult {
    fn concat(&mut self, other: Self) {
        self.diagnostics.extend(other.diagnostics);
        self.directives.extend(other.directives);
    }

    fn extend(&mut self, res: Result<Directive>) {
        match res {
            Ok(d) => self.directives.push(d),
            Err(d) => self.diagnostics.push(d),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirectiveError {
    pub diagnostic: Diagnostic,
    pub kind: DirectiveErrorKind,
}

impl DirectiveError {
    pub fn range(&self) -> Range<usize> {
        self.diagnostic.primary.as_ref().unwrap().span.range.clone()
    }

    pub fn new(diagnostic: Diagnostic, kind: DirectiveErrorKind) -> Self {
        Self { diagnostic, kind }
    }
}

#[derive(Debug, Clone)]
pub enum DirectiveErrorKind {
    ExpectedNotFound(Instruction),
    InvalidRule,
    InvalidCommandName,
    ExpectedCommand,
    Other,
}

pub struct DirectiveParser<'store, 'file> {
    /// The root node of a file, `SCRIPT` or `MODULE`.
    root: SyntaxNode,
    line_starts: Box<[usize]>,
    file: &'file File,
    store: Option<&'store CstRuleStore>,
    commands: Box<[CommandDescriptor]>,
    no_rewind: bool,
}

impl<'store, 'file> DirectiveParser<'store, 'file> {
    /// Create a new `DirectivesParser` with a root of a file which will
    /// use all default rules to check the rule names of a directive.
    ///
    /// # Panics
    ///
    /// If the given `root` is not `SCRIPT` or `MODULE`.
    pub fn new(root: SyntaxNode, file: &'file File) -> Self {
        Self::new_with_store(root, file, None)
    }

    /// Create a new `DirectivesParser` with a root of a file and a store of rules.
    ///
    /// # Panics
    ///
    /// If the given `root` is not `SCRIPT` or `MODULE`.
    pub fn new_with_store(
        root: SyntaxNode,
        file: &'file File,
        store: impl Into<Option<&'store CstRuleStore>>,
    ) -> Self {
        assert!(matches!(
            root.kind(),
            SyntaxKind::SCRIPT | SyntaxKind::MODULE
        ));

        Self {
            line_starts: line_starts(&root.to_string()).collect(),
            store: store.into(),
            root,
            file,
            no_rewind: false,
            commands: get_command_descriptors(),
        }
    }

    fn err(&self, msg: &str) -> Diagnostic {
        Diagnostic::error(self.file.id, "directives", msg)
    }

    fn line_of(&self, idx: usize) -> usize {
        self.line_starts
            .binary_search(&idx)
            .unwrap_or_else(|next_line| next_line - 1)
    }

    pub fn get_file_directives(&mut self) -> DirectiveResult {
        let top_level = self.top_level_directives();
        let mut result = DirectiveResult::default();

        for descendant in self.root.descendants().skip(1) {
            let comment = descendant
                .first_token()
                .and_then(|tok| tok.comment())
                .filter(|c| c.content.trim_start().starts_with(DECLARATOR));

            let comment = match comment {
                Some(comment) if comment.token.parent().is::<ModuleItem>() => comment,
                _ => continue,
            };

            let directive = self.parse_directive(comment, Some(descendant), false);
            result.extend(directive);
        }
        result.concat(top_level);
        result
    }

    pub fn top_level_directives(&mut self) -> DirectiveResult {
        let mut result = DirectiveResult::default();

        self.root
            .children_with_tokens()
            .flat_map(|item| item.into_token()?.comment())
            .filter(|comment| comment.content.trim_start().starts_with(DECLARATOR))
            .map(|comment| self.parse_directive(comment, None, true))
            .for_each(|res| result.extend(res));

        result
    }

    /// Parses a directive, based on all commands inside this `DirectivesParser`.
    fn parse_directive(
        &mut self,
        comment: Comment,
        node: Option<SyntaxNode>,
        top_level: bool,
    ) -> Result<Directive> {
        let text = comment
            .content
            .trim_start()
            .strip_prefix(DECLARATOR)
            .unwrap();

        let decl_offset = comment.content.len() - text.len();
        let offset = usize::from(comment.token.text_range().start()) + decl_offset + 1;

        let mut lexer = Lexer::new(text, self.file.id, offset);

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
            return Err(DirectiveError::new(d, DirectiveErrorKind::ExpectedCommand));
        }

        let cmd_tok = lexer.next().unwrap();
        let cmd_name = lexer.source_of(&cmd_tok);

        let cmd = self
            .commands
            .iter()
            .find(|cmd| cmd.name.eq_ignore_ascii_case(cmd_name))
            .map(|x| x.instructions.clone());

        let cmd = match cmd {
            Some(cmd) => cmd,
            None => {
                // TODO: Suggest name using `find_best_match_for_name`
                let d = self
                    .err(&format!("unknown directive command: `{}`", cmd_name))
                    .primary(cmd_tok.range, "");

                return Err(DirectiveError::new(
                    d,
                    DirectiveErrorKind::InvalidCommandName,
                ));
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

        let line = self.line_of(comment.token.text_range().start().into());
        Ok(Directive {
            // TODO: Report error for invalid command.
            command: Command::parse(&components, line, node, top_level, self.file),
            line,
            comment,
            components,
        })
    }

    fn parse_command(
        &mut self,
        lexer: &mut Lexer<'_>,
        first_component: Component,
        cmd: &[Instruction],
    ) -> Result<Vec<Component>> {
        self.no_rewind = false;
        let mut components = vec![first_component];

        for insn in &cmd[1..] {
            components.extend(self.parse_instruction(lexer, insn)?);
        }

        Ok(components)
    }

    fn parse_instruction(
        &mut self,
        lexer: &mut Lexer<'_>,
        insn: &Instruction,
    ) -> Result<Vec<Component>> {
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
                        self.no_rewind = true;
                        return Err(DirectiveError::new(d, DirectiveErrorKind::Other));
                    }
                    _ => {
                        let d = self.err("invalid number").primary(tok.range, "");
                        return Err(DirectiveError::new(d, DirectiveErrorKind::Other));
                    }
                };
                Ok(vec![Component {
                    kind: ComponentKind::Number(num),
                    range: tok.range,
                }])
            }
            Instruction::RuleName => {
                fn is_rule_name(kind: SyntaxKind) -> bool {
                    kind == T![-] || kind == T![ident] || kind.is_keyword()
                }

                let first = lexer
                    .next()
                    .filter(|tok| tok.kind != SyntaxKind::EOF)
                    .ok_or_else(|| {
                        let err = self
                            .err("expected rule name, but comment ends here")
                            .primary(lexer.abs_cur()..lexer.abs_cur() + 1, "");

                        DirectiveError::new(
                            err,
                            DirectiveErrorKind::ExpectedNotFound(Instruction::RuleName),
                        )
                    })?;
                if !is_rule_name(first.kind) {
                    let d = self
                        .err(&format!(
                            "expected `identifier`, `-` or `keyword`, but found `{}`",
                            format_kind(first.kind),
                        ))
                        .primary(first.range, "");
                    self.no_rewind = true;
                    return Err(DirectiveError::new(
                        d,
                        DirectiveErrorKind::ExpectedNotFound(Instruction::RuleName),
                    ));
                }
                let start = first.range.start();

                while lexer
                    .peek_with_spaces()
                    .map_or(false, |tok| is_rule_name(tok.kind))
                {
                    lexer.next();
                }

                let end = lexer.abs_cur() as u32;
                let name_range = TextRange::new(start, end.into());
                let name = lexer.source_range(name_range);

                let rule = self
                    .store
                    .map(|store| store.get(name))
                    .unwrap_or_else(|| crate::get_rule_by_name(name));
                if let Some(rule) = rule {
                    Ok(vec![Component {
                        kind: ComponentKind::Rule(rule),
                        range: name_range,
                    }])
                } else {
                    let mut d = self
                        .err(&format!("invalid rule: `{}`", name))
                        .primary(name_range, "");

                    if let Some(suggestion) = get_rule_suggestion(name) {
                        d = d.footer_help(format!("did you mean `{}`?", suggestion))
                    }
                    self.no_rewind = true;

                    Err(DirectiveError::new(d, DirectiveErrorKind::InvalidRule))
                }
            }
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
                    self.no_rewind = true;
                    Err(DirectiveError::new(
                        d,
                        DirectiveErrorKind::ExpectedNotFound(Instruction::Literal(lit)),
                    ))
                } else {
                    Ok(vec![Component {
                        kind: ComponentKind::Literal(lit),
                        range: tok.range,
                    }])
                }
            }
            Instruction::Optional(insns) => {
                let first = insns
                    .first()
                    .expect("every `Optional` instruction needs at least one element");
                if let Ok(first) = self.parse_instruction(lexer, first) {
                    let mut components = vec![];
                    components.extend(first);

                    for insn in insns.iter().skip(1) {
                        components.extend(self.parse_instruction(lexer, insn)?);
                    }

                    Ok(components)
                } else {
                    Ok(vec![])
                }
            }
            Instruction::Repetition(insn, separator) => {
                let mut first = true;
                let mut components = vec![];

                lexer.mark(true);
                let start = lexer.abs_cur() as u32;
                while lexer.peek().map_or(false, |tok| tok.kind == *separator) || first {
                    if !first {
                        lexer.expect(*separator)?;
                    }
                    let res = match self.parse_instruction(lexer, insn) {
                        Ok(res) => res,
                        // The first element isn't valid, so we continute with next instruction.
                        Err(_) if first && !self.no_rewind => {
                            lexer.mark(false);
                            lexer.rewind();
                            return Ok(vec![]);
                        }
                        err @ Err(_) => return err,
                    };
                    components.extend(res);

                    if first {
                        first = false;
                    }
                }
                lexer.mark(false);
                let end = lexer.abs_cur() as u32;

                Ok(vec![Component {
                    kind: ComponentKind::Repetition(components),
                    range: TextRange::new(start.into(), end.into()),
                }])
            }
            Instruction::Either(left, right) => self
                .parse_instruction(lexer, left)
                .or_else(|_| self.parse_instruction(lexer, right)),
        }
    }
}
