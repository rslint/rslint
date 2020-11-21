use crate::program::{Response, RunningProgram};
use std::num::NonZeroU64;

/// The id of a transaction
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TransactionId(NonZeroU64);

/// A handle to a ddlog transaction
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[must_use = "transactions can only be used with their transaction handles"]
pub struct TransactionHandle {
    id: TransactionId,
}

impl TransactionHandle {
    /// Create a new transaction handle
    #[doc(hidden)]
    pub const fn new(id: NonZeroU64) -> Self {
        Self {
            id: TransactionId(id),
        }
    }

    /// Turn the transaction handle into a u64
    #[doc(hidden)]
    pub const fn as_u64(&self) -> u64 {
        self.id.0.get()
    }

    /// Get the current handle's transaction id
    pub(super) const fn id(&self) -> &TransactionId {
        &self.id
    }
}

impl RunningProgram {
    /// Start a transaction.
    ///
    /// Fails if there is already a transaction in progress.
    pub fn transaction_start(&mut self) -> Response<TransactionHandle> {
        if self.transaction_in_progress.is_some() {
            return Err("transaction already in progress".to_string());
        }

        let id = TransactionId(self.transaction_counter);
        self.transaction_in_progress = Some(id);
        self.transaction_counter = NonZeroU64::new(self.transaction_counter.get() + 1)
            .expect("transaction_counter + 1 will always be non-zero");

        Ok(TransactionHandle { id })
    }

    /// Commit a transaction.
    pub fn transaction_commit(&mut self, transaction: TransactionHandle) -> Response<()> {
        if self.transaction_in_progress.is_none() {
            return Err("transaction_commit: no transaction in progress".to_string());
        // FIXME: Use `Option::contains()` https://github.com/rust-lang/rust/issues/62358
        } else if self.transaction_in_progress.as_ref() != Some(transaction.id()) {
            return Err("transaction_commit: invalid transaction handle given".to_string());
        }

        self.flush().and_then(|_| self.delta_cleanup()).map(|_| {
            self.transaction_in_progress = None;
        })
    }

    /// Rollback the transaction, undoing all changes.
    pub fn transaction_rollback(&mut self, transaction: TransactionHandle) -> Response<()> {
        if self.transaction_in_progress.is_none() {
            return Err("transaction_rollback: no transaction in progress".to_string());
        // FIXME: Use `Option::contains()` https://github.com/rust-lang/rust/issues/62358
        } else if self.transaction_in_progress.as_ref() != Some(transaction.id()) {
            return Err("transaction_rollback: invalid transaction handle given".to_string());
        }

        self.flush()
            .and_then(|_| self.delta_undo(&transaction))
            .map(|_| {
                self.transaction_in_progress = None;
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::Program;

    fn ddlog() -> RunningProgram {
        let prog: Program = Program {
            nodes: vec![],
            init_data: vec![],
        };

        prog.run(1).unwrap()
    }

    #[test]
    fn transaction_succeeds() {
        let mut ddlog = ddlog();
        let trans = ddlog.transaction_start().unwrap();
        ddlog.transaction_commit(trans).unwrap();
    }

    #[test]
    #[should_panic(expected = "transaction already in progress")]
    fn double_transaction_fails() {
        let mut ddlog = ddlog();
        let _ = ddlog.transaction_start().unwrap();
        let _ = ddlog.transaction_start().unwrap();
    }

    #[test]
    fn successive_transactions() {
        let mut ddlog = ddlog();
        let trans = ddlog.transaction_start().unwrap();
        ddlog.transaction_commit(trans).unwrap();

        let trans = ddlog.transaction_start().unwrap();
        ddlog.transaction_commit(trans).unwrap();
    }
}
