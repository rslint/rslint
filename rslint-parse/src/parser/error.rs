#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ParseDiagnosticType {
    MissingColonAfterKey,
    UnterminatedObjectLiteral,
    UnexpectedToken,
    ExpectedExpression,
    ExpectedIdentifier,
    InvalidRecovery,
    LinebreakInsidePostfixUpdate,
    InvalidTargetExpression,
    ConditionalWithoutColon,
    CommaWithoutRightExpression,
    ExpectedComma,
    UnmatchedBracket,
    ExpectedObjectKey,
    ExpectedSemicolon,
}
