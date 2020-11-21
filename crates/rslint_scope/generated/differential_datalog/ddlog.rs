use std::collections::btree_set::BTreeSet;
use std::fmt::Debug;
use std::iter::Iterator;
use std::ops::Deref;

use crate::{
    callback::Callback,
    ddval::DDValue,
    program::{IdxId, RelId, TransactionHandle, Update},
    record::{Record, UpdCmd},
    valmap::DeltaMap,
};

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Convert to and from values/objects of a DDlog program.
pub trait DDlogConvert: Debug {
    /// Convert a `RelId` into its symbolic name.
    fn relid2name(rel_id: RelId) -> Option<&'static str>;

    /// Convert a `IdxId` into its symbolic name.
    fn indexid2name(idx_id: IdxId) -> Option<&'static str>;

    /// Convert an `UpdCmd` into an `Update`.
    fn updcmd2upd(upd_cmd: &UpdCmd) -> Result<Update<DDValue>, String>;
}

/// A trait capturing program instantiation and handling of
/// transactions.
pub trait DDlog: Debug {
    type Convert: DDlogConvert;

    /// Wrapper type that implements Serialize/Deserialize functionality for Update<DDValue>.
    type UpdateSerializer: Debug
        + Send
        + Serialize
        + DeserializeOwned
        + From<Update<DDValue>>
        + Into<Update<DDValue>>;

    /// Run the program.
    fn run<F>(workers: usize, do_store: bool, cb: F) -> Result<(Self, DeltaMap<DDValue>), String>
    where
        Self: Sized,
        F: Callback;

    /// Start a transaction.
    fn transaction_start(&self) -> Result<TransactionHandle, String>;

    /// Commit a transaction previously started using
    /// `transaction_start`, producing a map of deltas.
    fn transaction_commit_dump_changes(
        &self,
        transaction: TransactionHandle,
    ) -> Result<DeltaMap<DDValue>, String>;

    /// Commit a transaction previously started using
    /// `transaction_start`.
    fn transaction_commit(&self, transaction: TransactionHandle) -> Result<(), String>;

    /// Roll back a transaction previously started using
    /// `transaction_start`.
    fn transaction_rollback(&self, transaction: TransactionHandle) -> Result<(), String>;

    /// Apply a set of updates.
    fn apply_updates<V, I>(&self, upds: I, transaction: &TransactionHandle) -> Result<(), String>
    where
        V: Deref<Target = UpdCmd>,
        I: Iterator<Item = V>;

    /// Apply a set of updates.
    fn apply_valupdates<I>(&self, upds: I, transaction: &TransactionHandle) -> Result<(), String>
    where
        I: Iterator<Item = Update<DDValue>>;

    /// Apply a set of updates directly from the flatbuffer
    /// representation
    #[cfg(feature = "flatbuf")]
    fn apply_updates_from_flatbuf(
        &self,
        buf: &[u8],
        transaction: &TransactionHandle,
    ) -> Result<(), String>;

    /// Query index.  Returns all values associated with the given key in the index.
    fn query_index(&self, index: IdxId, key: DDValue) -> Result<BTreeSet<DDValue>, String>;

    /// Query index passing key as a record.  Returns all values associated with the given key in the index.
    fn query_index_rec(&self, index: IdxId, key: &Record) -> Result<BTreeSet<DDValue>, String>;

    /// Similar to `query_index`, but extracts query from a flatbuffer.
    #[cfg(feature = "flatbuf")]
    fn query_index_from_flatbuf(&self, buf: &[u8]) -> Result<BTreeSet<DDValue>, String>;

    /// Dump all values in an index.
    fn dump_index(&self, index: IdxId) -> Result<BTreeSet<DDValue>, String>;

    /// Stop the program.
    fn stop(&mut self) -> Result<(), String>;
}
