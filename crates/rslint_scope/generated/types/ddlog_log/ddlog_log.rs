#![allow(
    path_statements,
    unused_imports,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_parens,
    non_shorthand_field_patterns,
    dead_code,
    overflowing_literals,
    unreachable_patterns,
    unused_variables,
    clippy::unknown_clippy_lints,
    clippy::missing_safety_doc,
    clippy::match_single_binding,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::needless_lifetimes,
    clippy::borrowed_box,
    clippy::map_clone,
    clippy::toplevel_ref_arg,
    clippy::double_parens,
    clippy::collapsible_if,
    clippy::clone_on_copy,
    clippy::unused_unit,
    clippy::deref_addrof,
    clippy::clone_on_copy,
    clippy::needless_return,
    clippy::op_ref,
    clippy::match_like_matches_macro,
    clippy::comparison_chain,
    clippy::len_zero,
    clippy::extra_unused_lifetimes
)]

use ::num::One;
use ::std::ops::Deref;

use ::differential_dataflow::collection;
use ::timely::communication;
use ::timely::dataflow::scopes;
use ::timely::worker;

//use ::serde::de::DeserializeOwned;
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::program::Weight;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;


// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;


/* Logging Configuration API, (detailed documentation in `ddlog_log.h`) */

use once_cell::sync::Lazy;
use std::collections;
use std::ffi;
use std::os::raw;
use std::sync;

type log_callback_t = Box<dyn Fn(i32, &str) + Send + Sync>;

struct LogConfig {
    default_callback: Option<log_callback_t>,
    default_level: i32,
    mod_callbacks: collections::HashMap<i32, (log_callback_t, i32)>,
}

impl LogConfig {
    fn new() -> LogConfig {
        LogConfig {
            default_callback: None,
            default_level: std::i32::MAX,
            mod_callbacks: collections::HashMap::new(),
        }
    }
}

/// Logger configuration for each module consists of the maximal enabled
/// log level (messages above this level are ignored) and callback.
static LOG_CONFIG: Lazy<sync::RwLock<LogConfig>> =
    Lazy::new(|| sync::RwLock::new(LogConfig::new()));

/// Logging API exposed to the DDlog program.
/// (see detailed documentation in `log.dl`)
#[allow(clippy::ptr_arg, clippy::trivially_copy_pass_by_ref)]
pub fn log(module: &i32, level: &i32, msg: &String) {
    let cfg = LOG_CONFIG.read().unwrap();
    if let Some((cb, current_level)) = cfg.mod_callbacks.get(&module) {
        if *level <= *current_level {
            cb(*level, msg.as_str());
        }
    } else if *level <= cfg.default_level && cfg.default_callback.is_some() {
        cfg.default_callback.as_ref().unwrap()(*level, msg.as_str());
    }
}

/// `cb = None` - disables logging for the given module.
// NOTE: we set callback and log level simultaneously.  A more flexible API
// would allow changing log level without changing the callback.
pub fn log_set_callback(module: i32, cb: Option<log_callback_t>, max_level: i32) {
    let mut cfg = LOG_CONFIG.write().unwrap();
    match cb {
        Some(cb) => {
            cfg.mod_callbacks.insert(module, (cb, max_level));
        }
        None => {
            cfg.mod_callbacks.remove(&module);
        }
    }
}

/// Set default callback and log level for modules that were not configured
/// via `log_set_callback`.
pub fn log_set_default_callback(cb: Option<log_callback_t>, max_level: i32) {
    let mut cfg = LOG_CONFIG.write().unwrap();
    cfg.default_callback = cb;
    cfg.default_level = max_level;
}

/// C bindings for the config API
#[no_mangle]
#[cfg(feature = "c_api")]
pub unsafe extern "C" fn ddlog_log_set_callback(
    module: raw::c_int,
    cb: Option<extern "C" fn(arg: libc::uintptr_t, level: raw::c_int, msg: *const raw::c_char)>,
    cb_arg: libc::uintptr_t,
    max_level: raw::c_int,
) {
    match cb {
        Some(cb) => log_set_callback(
            module as i32,
            Some(Box::new(move |level, msg| {
                cb(
                    cb_arg,
                    level as raw::c_int,
                    ffi::CString::new(msg).unwrap_or_default().as_ptr(),
                )
            })),
            max_level as i32,
        ),
        None => log_set_callback(module as i32, None, max_level as i32),
    }
}

#[no_mangle]
#[cfg(feature = "c_api")]
pub unsafe extern "C" fn ddlog_log_set_default_callback(
    cb: Option<extern "C" fn(arg: libc::uintptr_t, level: raw::c_int, msg: *const raw::c_char)>,
    cb_arg: libc::uintptr_t,
    max_level: raw::c_int,
) {
    match cb {
        Some(cb) => log_set_default_callback(
            Some(Box::new(move |level, msg| {
                cb(
                    cb_arg,
                    level as raw::c_int,
                    ffi::CString::new(msg).unwrap_or_default().as_ptr(),
                )
            })),
            max_level as i32,
        ),
        None => log_set_default_callback(None, max_level as i32),
    }
}
