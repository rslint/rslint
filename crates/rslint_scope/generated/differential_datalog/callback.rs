use crate::record::Record;

pub trait Callback: 'static + FnMut(usize, &Record, isize) + Clone + Send + Sync {}

impl<CB> Callback for CB where CB: 'static + FnMut(usize, &Record, isize) + Clone + Send + Sync {}
