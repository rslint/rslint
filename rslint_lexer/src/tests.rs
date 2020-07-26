use crate::Lexer;

macro_rules! assert_lex {
    ($src:expr, $($kind:ident:$len:expr $(,)?)*) => {{
        #[allow(unused_mut)]
        let mut lexer = Lexer::from_str($src, 0);
        let tokens = lexer.collect::<Vec<_>>();
        #[allow(unused_mut)]
        let mut idx = 0;

        $(
            assert_eq!(tokens[idx].0.kind, rslint_syntax::SyntaxKind::$kind,
                "expected token kind {}, but found {:?}", stringify!($kind), tokens[idx].0.kind
            );
            assert_eq!(tokens[idx].0.len, $len,
                "expected token length of {}, but found {} for token {:?}", $len, tokens[idx].0.len, tokens[idx].0.kind);
            idx += 1;
        )*

        assert_eq!(idx, tokens.len());
    }}
}

#[test]
fn empty() {
    assert_lex! {
        "",
    }
}

#[test]
fn identifier() {
    assert_lex! {
        "Abcdefg",
        IDENT:7
    }
}

#[test]
fn punctuators() {
    assert_lex! {
        "!%%&()*+,-.:;<=>?[]^{}|~",
        BANG:1,
        PERCENT:1,
        PERCENT:1,
        AMP:1,
        L_PAREN:1,
        R_PAREN:1,
        STAR:1,
        PLUS:1,
        COMMA:1,
        MINUS:1,
        DOT:1,
        COLON:1,
        SEMICOLON:1,
        L_ANGLE:1,
        EQ:1,
        R_ANGLE:1,
        QUESTION:1,
        L_BRACK:1,
        R_BRACK:1,
        CARET:1,
        L_CURLY:1,
        R_CURLY:1,
        PIPE:1,
        TILDE:1,
    }
}

#[test]
fn consecutive_punctuators() {
    assert_lex! {
        "&&&&^^^||",
        AMP:1,
        AMP:1,
        AMP:1,
        AMP:1,
        CARET:1,
        CARET:1,
        CARET:1,
        PIPE:1,
        PIPE:1,
    }
}

#[test]
fn unicode_whitespace() {
    assert_lex! {
        " \u{00a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}",
        WHITESPACE:17
    }
}

#[test]
fn unicode_whitespace_ident_part() {
    assert_lex! {
        "Abcd\u{2006}",
        IDENT:4,
        WHITESPACE:3 // length is in bytes
    }
}

#[test]
fn all_whitespace() {
    assert_lex! {
        "
         ",
        WHITESPACE:14
    }
}
