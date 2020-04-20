use clap::load_yaml;
use rslint::linter::Linter;
use rslint::linter::file_walker::FileWalker;

fn main() {
  env_logger::init();
  Linter::new(String::from("tests")).run();
}