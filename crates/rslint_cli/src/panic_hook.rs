//! The custom panic hook used by the linter to issue a more descriptive explanation.

use std::io::{self, Write};
use std::panic::PanicInfo;

pub fn panic_hook(info: &PanicInfo) {
    let stderr = io::stderr();

    let mut stderr_lock = stderr.lock();

    let mut write = |msg: &str| {
        writeln!(stderr_lock, "{}", msg).expect("panic_hook failed to write to stderr");
    };

    write("The linter panicked unexpectedly. This is a bug.\n");

    write("We would appreciate a bug report: https://github.com/rslint/rslint/issues/new?labels=ILE%2C+bug&template=internal-linter-error.md\n");

    write("Please include the following info: \n");

    let msg = info
        .payload()
        .downcast_ref::<String>()
        .map(|x| x.to_string())
        .unwrap_or_default();

    let location = info
        .location()
        .map(|l| format!("{}", l))
        .unwrap_or_default();

    write(format!("message: {}", msg).as_str());
    write(format!("location: {}", location).as_str());
    std::process::exit(-1);
}
