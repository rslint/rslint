//! Implementation of our own wrapper around the `FlameLayer`.

use std::{
    io::{BufWriter, Write},
    sync::mpsc::{Receiver, Sender},
};
use tracing_flame::FlameLayer;
use tracing_subscriber::Registry;

const FLAMEGRAPH_FILE: &str = "rslint.svg";

pub fn flame() -> (FlameGuard, FlameLayer<Registry, BufWriter<ChannelWriter>>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let write = BufWriter::new(ChannelWriter(tx));
    let flame = FlameLayer::new(write);
    let guard = FlameGuard { recv: rx };
    (guard, flame)
}

/// The guard will try to receive all bytes and then convert them
/// to a flamegraph which will then be outputted to a file if it is dropped.
pub struct FlameGuard {
    recv: Receiver<Vec<u8>>,
}

impl Drop for FlameGuard {
    fn drop(&mut self) {
        let string = self
            .recv
            .iter()
            .filter_map(|buf| String::from_utf8(buf).ok())
            .collect::<String>();

        let out = std::fs::File::create(FLAMEGRAPH_FILE).expect("failed to open flamegraph file");
        let mut out = BufWriter::new(out);

        inferno::flamegraph::from_lines(&mut Default::default(), string.lines(), &mut out)
            .expect("failed to generate flamegraph");
    }
}

/// A `Write` implementation that will send the received bytes through a channel.
///
/// It's recommended to wrap this type into a `BufWriter`.
pub struct ChannelWriter(Sender<Vec<u8>>);

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .send(buf.to_vec())
            .expect("failed to send data through channel");
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
