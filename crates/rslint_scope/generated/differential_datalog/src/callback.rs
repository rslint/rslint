use crate::record::Record;

pub trait Callback: 'static + Fn(usize, &Record, isize) + Clone + Send + Sync {}

impl<CB> Callback for CB where CB: 'static + Fn(usize, &Record, isize) + Clone + Send + Sync {}
