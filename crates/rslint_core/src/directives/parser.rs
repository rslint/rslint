use crate::{util::find_best_match_for_name, CstRule, CstRuleStore, Diagnostic, DiagnosticBuilder};
use codespan_reporting::diagnostic::Severity;
use rslint_lexer::Lexer as RawLexer;
use rslint_parser::{
    util::Comment, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTokenExt, TextRange, T,
};
use std::collections::HashMap;
use std::iter::Peekable;
use std::ops::Range;

pub struct DirectiveParseResult {
    pub diagnostics: Vec<Diagnostic>,
    pub directive: Directive,
}

#[derive(Debug, Clone)]
pub enum Command {
    /// Ignore linting for the entire file.
    IgnoreFile,
    /// Ignore one or more rules on a node.
    IgnoreRules(Vec<Box<dyn CstRule>>, TextRange),
    /// Ignore any rules on a node.
    // We cannot store the actual node because Nodes are !Sync and !Send because
    // they are a wrapper around an Rc<NodeData>
    IgnoreNode(TextRange),
    /// Ignore rules for an entire file.
    IgnoreRulesFile(Vec<Box<dyn CstRule>>),
}

impl Command {
    /// Whether this command applies to the entire file.
    pub fn top_level(&self) -> bool {
        matches!(self, Command::IgnoreFile | Command::IgnoreRulesFile(_))
    }
}

/// A command given to the linter by an inline comment.
/// A single command may include multiple commands inside of it.
/// A directive constitutes a single comment, which may have one or more commands inside of it.
#[derive(Debug, Clone)]
pub struct Directive {
    pub commands: Vec<Command>,
    pub comment: Comment,
}

struct RawCommand {
    tokens: Vec<Token>,
    // partially incomplete (rule vectors)
    kind: Command,
}

struct RawDirective {
    commands: Vec<RawCommand>,
    comment: Comment,
}

pub struct DirectiveParser<'store> {
    pub root_node: SyntaxNode,
    /// A string denoting the start of a directive, `rslint-` by default.
    pub declarator: String,
    file_id: usize,
    store: &'store CstRuleStore,
}

impl<'store> DirectiveParser<'store> {
    /// Make a new directive parser from the root node of a file.
    ///
    /// # Panics
    /// Panics if the node's kind is not SCRIPT or MODULE
    pub fn new(root_node: SyntaxNode, file_id: usize, store: &'store CstRuleStore) -> Self {
        assert!(matches!(
            root_node.kind(),
            SyntaxKind::SCRIPT | SyntaxKind::MODULE
        ));

        Self {
            root_node,
            declarator: "rslint-".to_string(),
            file_id,
            store,
        }
    }

    pub fn get_file_directives(&self) -> Result<Vec<DirectiveParseResult>, Diagnostic> {
        let mut raw = self.extract_top_level_directives()?;
        // descendants yields the root node first, so we need to skip it
        for descendant in self.root_node.descendants().skip(1) {
            if let Some(comment) = descendant.first_token().and_then(|tok| tok.comment()) {
                if comment.content.trim_start().starts_with(&self.declarator) {
                    let commands = self.parse_directive(comment.token.clone(), Some(descendant))?;
                    raw.push(RawDirective { comment, commands });
                }
            }
        }

        Ok(raw
            .into_iter()
            .map(|raw| self.bake_raw_directive(raw))
            .collect())
    }

    fn err(&self, message: impl AsRef<str>) -> DiagnosticBuilder {
        DiagnosticBuilder::error(self.file_id, "directives", message.as_ref())
    }

    fn bake_raw_directive(&self, directive: RawDirective) -> DirectiveParseResult {
        let mut diagnostics = vec![];
        let mut commands = vec![];

        for raw_command in directive.commands.into_iter() {
            let (diags, rules) = self.bake_ignore_command(&raw_command);
            diagnostics.extend(diags);
            let command = match raw_command.kind {
                Command::IgnoreFile | Command::IgnoreNode(_) => raw_command.kind,
                Command::IgnoreRules(_, node) => Command::IgnoreRules(rules, node),
                Command::IgnoreRulesFile(_) => Command::IgnoreRulesFile(rules),
            };
            commands.push(command);
        }
        let directive = Directive {
            commands,
            comment: directive.comment,
        };

        DirectiveParseResult {
            directive,
            diagnostics,
        }
    }

    fn bake_ignore_command(
        &self,
        command: &RawCommand,
    ) -> (Vec<Diagnostic>, Vec<Box<dyn CstRule>>) {
        let mut unique: HashMap<&String, &Range<usize>> =
            HashMap::with_capacity(command.tokens.len());
        let mut diagnostics = vec![];
        let mut rules = Vec::with_capacity(command.tokens.len());

        for Token { range, raw } in command.tokens.iter() {
            if let Some(prev_range) = unique.get(raw) {
                let warn = self
                    .err("redundant duplicate rules in `ignore` directive")
                    .severity(Severity::Warning)
                    .secondary(
                        prev_range.to_owned().to_owned(),
                        format!("{} is ignored here", raw),
                    )
                    .primary(range.clone(), "this ignore is redundant");

                diagnostics.push(warn.into());
            } else {
                unique.insert(raw, range);
            }

            if let Some(rule) = CstRuleStore::new().builtins().get(raw) {
                if self.store.get(raw).is_none() {
                    let warn = self
                        .err(format!(
                            "redundant rule in `ignore` directive, `{}` is already allowed",
                            raw
                        ))
                        .severity(Severity::Warning)
                        .primary(range.to_owned(), "");

                    diagnostics.push(warn.into());
                } else {
                    rules.push(rule);
                }
            } else {
                let mut err = self
                    .err(format!("unknown rule `{}` used in directive", raw))
                    .primary(range.to_owned(), "");

                if let Some(suggestion) = find_best_match_for_name(
                    CstRuleStore::new()
                        .builtins()
                        .rules
                        .iter()
                        .map(|x| x.name()),
                    raw,
                    None,
                ) {
                    err = err.note(format!("help: did you mean `{}`?", suggestion));
                }
                diagnostics.push(err.into());
            }
        }
        (diagnostics, rules)
    }

    /// Extract directives which apply to the whole file such as `rslint-ignore` or `rslint-ignore rule`.
    fn extract_top_level_directives(&self) -> Result<Vec<RawDirective>, Diagnostic> {
        let comments: Vec<Comment> = self
            .root_node
            .children_with_tokens()
            .scan((), |_, item| {
                item.into_token().filter(|tok| tok.kind().is_trivia())
            })
            .filter(|t| {
                t.kind() == SyntaxKind::COMMENT
                    && t.comment()
                        .unwrap()
                        .content
                        .trim_start()
                        .starts_with(&self.declarator)
            })
            .map(|token| token.comment().unwrap())
            .collect();

        self.parse_comments(comments)
    }

    fn parse_comments(&self, comments: Vec<Comment>) -> Result<Vec<RawDirective>, Diagnostic> {
        let mut directives = Vec::with_capacity(comments.len());
        for comment in comments {
            let commands = self.parse_directive(comment.token.clone(), None)?;
            directives.push(RawDirective { commands, comment });
        }
        Ok(directives)
    }

    fn parse_directive(
        &self,
        comment: SyntaxToken,
        node: Option<SyntaxNode>,
    ) -> Result<Vec<RawCommand>, Diagnostic> {
        let inner_text = comment.comment().unwrap().content;
        let stripped_text = inner_text
            .trim_start()
            .strip_prefix(&self.declarator)
            .unwrap();
        let declaration_offset = comment.text().len() - inner_text.len();
        let offset = usize::from(comment.text_range().start())
            + (inner_text.trim_start().len() - stripped_text.len())
            + declaration_offset
            + 1;
        let string = self.root_node.to_string();
        let mut lexer = Lexer::new(stripped_text, offset, self.file_id, string.as_str());

        let mut first = true;
        let mut raw_commands = vec![];

        while !lexer
            .peek_no_whitespace()
            .map_or(false, |t| t.kind == T![--] || t.kind == SyntaxKind::EOF)
        {
            if first {
                first = false;
            } else if lexer.peek_no_whitespace().map(|x| x.kind) != Some(T![-]) {
                return Err(self
                    .err("Directive commands must be separated by `-`")
                    .primary(
                        lexer.cur..lexer.cur + lexer.peek_no_whitespace().unwrap().len,
                        "",
                    )
                    .into());
            } else {
                lexer.next();
            }

            raw_commands.push(self.parse_command(&mut lexer, node.clone())?);
        }
        Ok(raw_commands)
    }

    /// Parse a single command and advance the token source accordingly.
    fn parse_command(
        &self,
        lexer: &mut Lexer,
        node: Option<SyntaxNode>,
    ) -> Result<RawCommand, Diagnostic> {
        let word = lexer.word()?;
        match word.raw.as_str() {
            "ignore" => {
                if lexer
                    .peek_no_whitespace()
                    .map(|t| t.kind)
                    .filter(|kind| kind == &T![ident] || kind.is_keyword())
                    .is_some()
                {
                    let tokens = lexer.rule_list()?;
                    let kind = if let Some(node) = node {
                        Command::IgnoreRules(vec![], node.text_range())
                    } else {
                        Command::IgnoreRulesFile(vec![])
                    };

                    Ok(RawCommand { tokens, kind })
                } else {
                    let kind = if let Some(node) = node {
                        Command::IgnoreNode(node.text_range())
                    } else {
                        Command::IgnoreFile
                    };

                    Ok(RawCommand {
                        tokens: vec![],
                        kind,
                    })
                }
            }
            text => {
                const COMMANDS: [&str; 1] = ["ignore"];

                let mut err = self
                    .err(format!("unknown directive command `{}`", text))
                    .primary(word.range, "");

                if let Some(suggestion) =
                    find_best_match_for_name(COMMANDS.iter().cloned(), text, None)
                {
                    err = err.note(format!("help: did you mean `{}`", suggestion));
                }
                Err(err.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Token {
    range: Range<usize>,
    raw: String,
}

#[derive(Debug, Clone)]
struct Lexer<'src> {
    offset: usize,
    // we just reuse rslint_lexer which takes care of the dirty work for us
    raw: Peekable<RawLexer<'src>>,
    src: &'src str,
    pub cur: usize,
    file_id: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str, offset: usize, file_id: usize, full_src: &'src str) -> Lexer<'src> {
        Self {
            offset,
            raw: RawLexer::from_str(src, file_id).peekable(),
            src: full_src,
            cur: offset,
            file_id,
        }
    }

    fn next(&mut self) -> Option<rslint_lexer::Token> {
        let next = self.raw.next();
        if let Some((tok, _)) = next {
            self.cur += tok.len;
            if tok.kind.is_trivia() {
                return self.next();
            }
            Some(tok)
        } else {
            None
        }
    }

    pub fn peek_no_whitespace(&mut self) -> Option<rslint_lexer::Token> {
        let peeked = self.raw.peek();
        if let Some((tok, _)) = peeked {
            if tok.kind.is_trivia() {
                self.raw.next();
                return self.peek();
            }
            Some(*tok)
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<rslint_lexer::Token> {
        self.raw.peek().map(|(t, _)| t).cloned()
    }

    fn err(&self, msg: impl AsRef<str>) -> DiagnosticBuilder {
        DiagnosticBuilder::error(self.file_id, "linter", msg.as_ref())
    }

    fn range(&self, token: rslint_lexer::Token) -> Range<usize> {
        self.cur - token.len..self.cur
    }

    fn range_inclusive(&self, token: rslint_lexer::Token) -> Range<usize> {
        self.cur - token.len..self.cur + 1
    }

    pub fn word(&mut self) -> Result<Token, Diagnostic> {
        let end = self.src.len() + self.offset;
        let next: rslint_lexer::Token = self.next().ok_or_else(|| {
            self.err("Expected a word when parsing a directive, but the comment ends prematurely")
                .primary(end..end + 1, "comment ends here")
                .finish()
        })?;
        if next.kind != T![ident] && !next.kind.is_keyword() {
            return Err(self
                .err("Expected a word when parsing a directive, but found none")
                .primary(self.range(next), "expected a word here")
                .into());
        }
        let range = self.range(next);

        Ok(Token {
            range: range.clone(),
            raw: self.src[range].to_string(),
        })
    }

    pub fn rule_name(&mut self) -> Result<Token, Diagnostic> {
        let end = self.src.len() + self.offset;
        let next = self.next().ok_or_else(|| {
            self.err(
                "Expected a rule name when parsing a directive, but the comment ends prematurely",
            )
            .primary(end..end + 1, "comment ends here")
            .finish()
        })?;
        let start = self.range(next).start + 1;
        let mut tok = next;

        loop {
            if self.peek().map(|tok| tok.kind) == Some(T![-]) {
                tok = self.next().unwrap();
                let kind = self.peek().map(|t| t.kind);
                if kind == Some(T![ident]) || kind.map_or(false, |kind| kind.is_keyword()) {
                    tok = self.next().unwrap();
                    continue;
                } else {
                    let range = start..self.range_inclusive(tok).end;
                    return Ok(Token {
                        range: range.clone(),
                        raw: self.src[range].to_string(),
                    });
                }
            } else {
                let range = start..self.range_inclusive(tok).end;
                return Ok(Token {
                    range: range.clone(),
                    raw: self.src[range].to_string(),
                });
            }
        }
    }

    pub fn rule_list(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut toks = vec![];

        toks.push(self.rule_name()?);
        loop {
            if self.peek_no_whitespace().map(|t| t.kind) == Some(T![,]) {
                self.next();
                toks.push(self.rule_name()?);
            } else {
                return Ok(toks);
            }
        }
    }
}
