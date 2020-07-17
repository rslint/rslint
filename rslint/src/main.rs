use rslint::linter::Linter;

fn main() {
  env_logger::init();
  Linter::from_cli_args().run();
}
