#![allow(
    path_statements,
    //unused_imports,
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
    clippy::match_single_binding
)]

// Required for #[derive(Serialize, Deserialize)].
use ::serde::Deserialize;
use ::serde::Serialize;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;

use crate::string_append_str;
use crate::string_append;
use crate::std_usize;
use crate::closure;

//
// use crate::ddlog_std;

/// Allow emitting debug messages from within datalog
// TODO: Replace with tracing
pub fn debug(message: &String) {
    println!("[datalog debug]: {}", message);
}

/* fn debug(message: & String) -> () */
pub fn or_else<T: crate::Val>(option: & crate::ddlog_std::Option<T>, option_b: & crate::ddlog_std::Option<T>) -> crate::ddlog_std::Option<T>
{   match (*option) {
        crate::ddlog_std::Option::Some{x: _} => (*option).clone(),
        crate::ddlog_std::Option::None{} => (*option_b).clone()
    }
}