pub mod parse;
pub mod linter;
pub mod macros;

use clap::App;
use clap::load_yaml;
use crate::linter::Linter;

fn main() {
  let yaml = load_yaml!("../cli.yml");
  Linter::new(String::from("tests/main.js")).unwrap().run();
}