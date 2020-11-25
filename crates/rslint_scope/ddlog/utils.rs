use std::fmt::Debug;

pub fn dbg<T: Debug>(val: T) {
    tracing::trace!(target: "ddlog", "{:#?}", &val);
}
