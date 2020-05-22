#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ParseDiagnosticType {
    MissingColonAfterKey,
    InvalidCommaInsideObject,
    UnterminatedObjectLiteral,
    UnexpectedToken,
    ExpectedExpression,
    ExpectedIdentifier,
    InvalidRecovery,
    LinebreakInsidePostfixUpdate,
    InvalidTargetExpression,
}
