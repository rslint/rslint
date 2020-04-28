use std::hash::Hash;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum LexerDiagnosticType {
  // Errors
  UnterminatedString,
  UnexpectedToken,
  TemplateLiteralInEs5,
  UnterminatedMultilineComment,
  IdentifierStartAfterNumber,
  MultipleExponentsInNumber,
  IncompleteExponent,
  DecimalExponent,
  PeriodInFloat,
  UnterminatedRegex,
  InvalidRegexFlags,
  InvalidCharAfterZero,
  InvalidOctalLiteral,
  InvalidHexCharacter,
  MissingHexDigit,

  // Notes
  RedundantZeroesAfterNumber,
  RedundantExponent,
}