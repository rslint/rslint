#[macro_export]
macro_rules! keyword_trie {
  ($lexer:expr, $target:expr, $start:expr, {$($body:tt)*}) => {
    {
        keyword_trie!{  $lexer, $target, $start, {}, $($body)*, true }
    }
  };
  ( $lexer:expr, $target:expr, $start:expr, {$($arms:tt)*}, $(,)*, $first:expr) => {
    match Some($target) {
      $($arms)*
      _ => $lexer.resolve_identifier($start)
    }
  };
  ( $lexer:expr, $target:expr, $start:expr, {$($arms:tt)*}, $(,)*) => {
    match $lexer.advance() {
        $($arms)*
        _ => $lexer.resolve_identifier($start)
    }
  };
  ( $lexer:expr, $target:expr, $start:expr, {$($arms:tt)*}, _ => $e:expr $(,)*) => {
    match $lexer.advance() {
        $($arms)*
        _ => $lexer.resolve_identifier($start)
    }
  };
  ( $lexer:expr, $target:expr, $start:expr, {$($arms:tt)*}, $p:expr => { $($block:tt)* }, $($tail:tt)*) => {
    keyword_trie!{
         $lexer, $target, $start,
        {
            $($arms)*
            Some($p) => {
              keyword_trie!( $lexer, $target, $start, {}, $($block)*)
            },
        },
        $($tail)*
    }
  };
  ( $lexer:expr, $target:expr, $start:expr, {$($arms:tt)*}, $p:expr => $expected_token:ident, $($tail:tt)*) => {
    keyword_trie!{
         $lexer, $target, $start,
        {
            $($arms)*
            Some($p) => {
              $lexer.resolve_keyword(TokenType::$expected_token, $start, &stringify!($expected_token).to_ascii_lowercase())
            },
        },
        $($tail)*
    }
  };
}
