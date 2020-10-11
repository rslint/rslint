//! The custom panic hook used by the linter to issue a more descriptive explanation.

use crate::{lint_err, lint_note};
use std::panic::PanicInfo;
use std::io::{stderr, Write};

pub fn panic_hook(info: &PanicInfo) {
    lint_err!("The linter panicked unexpectedly. this is a bug.");

    let stderr = stderr();
    let mut stderr_lock = stderr.lock();

    writeln!(stderr_lock, "We would appreciate a bug report: https://github.com/RDambrosio016/RSLint/issues/new?labels=ILE%2C+bug&template=internal-linter-error.md\n").expect("panic_hook failed to write to stderr");

    lint_note!("Please include the following info: ");

    let msg = info
        .payload()
        .downcast_ref::<String>()
        .map(|x| x.to_string())
        .unwrap_or_default();

    let location = info.location()
            .map(|l| format!("{}", l))
            .unwrap_or_default();

    writeln!(stderr_lock, "message: {}", msg).expect("panic_hook failed to write to stderr");
    writeln!(stderr_lock, "location: {}", location).expect("panic_hook failed to write to stderr");
}
