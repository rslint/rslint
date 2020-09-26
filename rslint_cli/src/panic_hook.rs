//! The custom panic hook used by the linter to issue a more descriptive explanation.

use std::panic::PanicInfo;
use crate::{lint_err, lint_note};

pub fn panic_hook(info: &PanicInfo) {
    lint_err!("The linter panicked unexpectedly. this is a bug.");
    eprintln!("We would appreciate a bug report: https://github.com/RDambrosio016/RSLint/issues/new?labels=ILE%2C+bug&template=internal-linter-error.md\n");
    lint_note!("Please include the following info: ");

    let msg = info.payload().downcast_ref::<String>().map(|x| x.to_string()).unwrap_or_default();
    eprintln!("message: {}", msg);
    eprintln!("location: {}", info.location().map(|l| format!("{}", l)).unwrap_or_default());
}
