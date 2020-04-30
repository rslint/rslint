use std::hash::Hash;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum LexerDiagnosticType {
  // Errors
  UnterminatedString,
  UnexpectedToken,
  TemplateLiteralInEs5,
  UnterminatedMultilineComment,
  IdentifierStartAfterNumber,
  IncompleteExponent,
  DecimalExponent,
  PeriodInFloat,
  UnterminatedRegex,
  InvalidRegexFlags,
  InvalidCharAfterZero,
  InvalidOctalLiteral,
  InvalidHexCharacter,
  MissingHexDigit,
  InvalidUnicodeEscapeSequence,
  InvalidUnicodeIdentStart,
  IncompleteUnicodeEscapeSequence,
  InvalidHexEscapeSequence,
  IncompleteHexEscapeSequence,

  // Notes
  RedundantZeroesAfterNumber,
  RedundantExponent,
  RedundantHexZeroes,
}