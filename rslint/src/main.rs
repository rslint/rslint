use rslint::linter::Linter;

fn main() {
  env_logger::init();
  Linter::new("tests/main.js".to_string()).run();
}
