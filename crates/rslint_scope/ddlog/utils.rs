use std::fmt::Debug;

/// Allow emitting debug messages from within datalog
// TODO: Replace with tracing
pub fn debug(message: &String) {
    println!("[datalog debug]: {}", message);
}

pub fn dbg<T: Debug>(val: T) {
    println!("[datalog debug]: {:#?}", &val);
}
