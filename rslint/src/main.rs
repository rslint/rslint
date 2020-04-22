use rslint::linter::Linter;

fn main() {
  env_logger::init();
  Linter::new(String::from("tests/main.js")).run()
    .expect("Failed to run linter");
}