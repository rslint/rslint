
pub struct FileContext {
  disabled: bool,
  ruleOverrides: Vec<Rule>,
}

// Temp struct for representing a rule
pub struct Rule {
  state: u8,
  name: &str,
}