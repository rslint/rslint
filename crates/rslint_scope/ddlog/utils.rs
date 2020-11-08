/// Allow emitting debug messages from within datalog
// TODO: Replace with tracing
pub fn debug(message: &String) {
    println!("[datalog debug]: {}", message);
}
