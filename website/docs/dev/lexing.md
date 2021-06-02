# Lexing

Lexing is the first step which occurs in parsing ECMAScript. The job of a lexer is to take raw source code and turn it into abstract tokens which the parser then works on. RSLint uses its own lexer implementation in [`rslint_lexer`](https://github.com/rslint/rslint/tree/master/crates/rslint_lexer/src).

For example, code such as `var a = 2 + 2;` will be broken down into:

```
VAR_KW
WHITESPACE
IDENT
WHITESPACE
EQ
WHITESPACE
NUMBER
WHITESPACE
PLUS
WHITESPACE
NUMBER
SEMICOLON
```

As you probably noticed, whitespace is included in the tokens for reasons we will cover later. Although, the parser does not work on tokens with whitespace and comments (whitespace and comments are sometimes called trivia), there is an intermediate structure called the [`TokenSource`](https://github.com/rslint/rslint/blob/master/crates/rslint_parser/src/token_source.rs) which manages the raw tokens and turning them into tokens the parser can use.

## Losslessness

The lexer is fully lossless, which means the tokens produced fully represent the original source code. However, the tokens do not keep any kind of source code, tokens are a very simple structure:

```rust
pub struct Token {
    /// The kind of token this is.
    pub kind: SyntaxKind,
    /// How long the token is in bytes.
    pub len: usize,
}
```

`SyntaxKind` is an enum you will see used everywhere in RSLint, it is a single unified enum which represents the kinds of all of the following:

- All possible token kinds
- All possible AST node kinds

This central concept of a single enum is important because RSLint's syntax tree is deeply intertwined with the underlying tokens as you will see later on. The [`SyntaxKind`](https://github.com/rslint/rslint/blob/master/crates/rslint_syntax/src/generated.rs) enum is not written manually, it is generated from [a file](https://github.com/rslint/rslint/blob/master/xtask/src/ast.rs) which also houses AST definitions which generate AST structs as you will see later.
