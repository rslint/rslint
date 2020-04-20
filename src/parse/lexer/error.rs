use std::hash::Hash;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum LexerDiagnosticType {
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
}