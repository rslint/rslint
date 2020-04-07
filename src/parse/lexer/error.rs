use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::hash::Hash;
use std::stringify;
use crate::parse::span::Span;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum LexerErrors {
  UnterminatedString,
  UnexpectedToken
}

#[derive(Debug)]
pub struct LexerDiagnostic {
  pub diagnostic: Diagnostic<usize>
}

macro_rules! error_map {
  ($map:expr, $($identifier:ident => $error:expr),* $(,)?) => {
    $($map.insert(LexerErrors::$identifier, $error);)*
  };
}

static LEXER_ERROR_MAP: Lazy<HashMap<LexerErrors, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    error_map!(map,
      UnexpectedToken => "Unexpected token",
      UnterminatedString => "Unterminated string literal",
    );
    map
  }
);

impl LexerDiagnostic {

  fn new(file_id: usize,
    error: LexerErrors,
    primary_labels: Option<Vec<(Span, &str)>>, 
    secondary_labels: Option<Vec<(Span, &str)>>
  ) -> Diagnostic<usize> {

    Diagnostic::error()
      .with_code(stringify!(error))
      .with_message(*LEXER_ERROR_MAP.get(&error).unwrap())
      .with_labels(primary_labels.unwrap_or(vec![])
        .iter().map(|label| Label::primary(file_id, label.0.range())
          .with_message(label.1)
        )
        .collect::<Vec<Label<usize>>>()
      )
      .with_labels(secondary_labels.unwrap_or(vec![])
        .iter().map(|label| Label::secondary(file_id, label.0.range())
          .with_message(label.1)
        )
        .collect::<Vec<Label<usize>>>()
      )
  }
}
