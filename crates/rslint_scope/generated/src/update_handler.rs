//! `UpdateHandler` trait and its implementations.
//!
//! `UpdateHandler` abstracts away different methods of handling
//! relation update notifications from differential dataflow.
//! Possible implementations include:
//! - handling notifications by invoking a user-defined callback
//! - storing output tables in an in-memory database
//! - accumulating changes from one or multiple transactions in
//!   an in-memory database
//! - chaining multiple update handlers
//! - all of the above, but processed by a separate thread
//!   rather than the differential worker threads that computes
//!   the update
//! - all of the above, but processed by a pool of worker threads

use super::*;

use std::cell::Cell;
use std::fmt::{self, Debug, Formatter};
use std::sync::mpsc::*;
use std::sync::{Arc, Barrier, Mutex, MutexGuard};
use std::thread::*;

use differential_datalog::program::CBFn;
use differential_datalog::program::RelId;
use differential_datalog::Callback;
use differential_datalog::DeltaMap;

/// Single-threaded (non-thread-safe callback)
pub trait ST_CBFn: FnMut(RelId, &DDValue, isize) {
    fn clone_boxed(&self) -> Box<dyn ST_CBFn>;
}

impl<T> ST_CBFn for T
where
    T: 'static + Clone + FnMut(RelId, &DDValue, isize),
{
    fn clone_boxed(&self) -> Box<dyn ST_CBFn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ST_CBFn> {
    fn clone(&self) -> Self {
        self.as_ref().clone_boxed()
    }
}

pub trait UpdateHandler: Debug {
    /// Returns a handler to be invoked on each output relation update.
    fn update_cb(&self) -> Box<dyn ST_CBFn>;

    /// Notifies the handler that a transaction_commit method is about to be
    /// called. The handler has an opportunity to prepare to handle
    /// update notifications.
    fn before_commit(&self);

    /// Notifies the handler that transaction_commit has finished. The
    /// `success` flag indicates whether the commit succeeded or failed.
    fn after_commit(&self, success: bool);
}

/// Multi-threaded update handler that can be invoked from multiple DDlog
/// worker threads.
pub trait MTUpdateHandler: UpdateHandler + Sync + Send {
    /// Returns a thread-safe handler to be invoked on each output
    /// relation update.
    fn mt_update_cb(&self) -> Box<dyn CBFn>;
}

/// Rust magic to make `MTUpdateHandler` clonable.
pub trait IMTUpdateHandler: MTUpdateHandler {
    fn clone_boxed(&self) -> Box<dyn IMTUpdateHandler>;
}

impl<T> IMTUpdateHandler for T
where
    T: MTUpdateHandler + Clone + 'static,
{
    fn clone_boxed(&self) -> Box<dyn IMTUpdateHandler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn IMTUpdateHandler> {
    fn clone(&self) -> Self {
        self.as_ref().clone_boxed()
    }
}

/// A no-op `UpdateHandler` implementation
#[derive(Clone, Copy, Debug, Default)]
pub struct NullUpdateHandler {}

impl NullUpdateHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl UpdateHandler for NullUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        Box::new(|_, _, _| {})
    }
    fn before_commit(&self) {}
    fn after_commit(&self, _success: bool) {}
}

impl MTUpdateHandler for NullUpdateHandler {
    fn mt_update_cb(&self) -> Box<dyn CBFn> {
        Box::new(|_, _, _| {})
    }
}

/// `UpdateHandler` implementation that invokes user-provided closure.
#[derive(Clone)]
pub struct CallbackUpdateHandler<F: Callback> {
    cb: F,
}

impl<F: Callback> CallbackUpdateHandler<F> {
    pub fn new(cb: F) -> Self {
        Self { cb }
    }
}

impl<F: Callback> Debug for CallbackUpdateHandler<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("CallbackUpdateHandler");
        let _ = builder.field("cb", &(&self.cb as *const F));
        builder.finish()
    }
}

impl<F: Callback> UpdateHandler for CallbackUpdateHandler<F> {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let mut cb = self.cb.clone();
        Box::new(move |relid, v, w| cb(relid, &v.clone().into_record(), w))
    }
    fn before_commit(&self) {}
    fn after_commit(&self, _success: bool) {}
}

impl<F: Callback> MTUpdateHandler for CallbackUpdateHandler<F> {
    fn mt_update_cb(&self) -> Box<dyn CBFn> {
        let mut cb = self.cb.clone();
        Box::new(move |relid, v, w| cb(relid, &v.clone().into_record(), w as isize))
    }
}

#[cfg(feature = "c_api")]
pub type ExternCCallback = extern "C" fn(
    arg: libc::uintptr_t,
    table: libc::size_t,
    rec: *const record::Record,
    weight: libc::ssize_t,
);

/// `UpdateHandler` implementation that invokes user-provided C function.
#[cfg(feature = "c_api")]
#[derive(Clone, Copy, Debug)]
pub struct ExternCUpdateHandler {
    cb: ExternCCallback,
    cb_arg: libc::uintptr_t,
}

#[cfg(feature = "c_api")]
impl ExternCUpdateHandler {
    pub fn new(cb: ExternCCallback, cb_arg: libc::uintptr_t) -> Self {
        Self { cb, cb_arg }
    }
}

#[cfg(feature = "c_api")]
impl UpdateHandler for ExternCUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let cb = self.cb;
        let cb_arg = self.cb_arg;
        Box::new(move |relid, v, w| {
            cb(
                cb_arg,
                relid,
                &v.clone().into_record() as *const record::Record,
                w,
            )
        })
    }
    fn before_commit(&self) {}
    fn after_commit(&self, _success: bool) {}
}

#[cfg(feature = "c_api")]
impl MTUpdateHandler for ExternCUpdateHandler {
    fn mt_update_cb(&self) -> Box<dyn CBFn> {
        let cb = self.cb;
        let cb_arg = self.cb_arg;
        Box::new(move |relid, v, w| {
            cb(
                cb_arg,
                relid,
                &v.clone().into_record() as *const record::Record,
                w as isize,
            )
        })
    }
}

/// Multi-threaded `UpdateHandler` implementation that stores updates
/// in a `DeltaMap` and locks the map on every update.
#[derive(Clone, Debug)]
pub struct MTValMapUpdateHandler {
    db: Arc<Mutex<DeltaMap<DDValue>>>,
}

impl MTValMapUpdateHandler {
    pub fn new(db: Arc<Mutex<DeltaMap<DDValue>>>) -> Self {
        Self { db }
    }
}

impl UpdateHandler for MTValMapUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let db = self.db.clone();
        Box::new(move |relid, v, w| db.lock().unwrap().update(relid, v, w))
    }
    fn before_commit(&self) {}
    fn after_commit(&self, _success: bool) {}
}

impl MTUpdateHandler for MTValMapUpdateHandler {
    fn mt_update_cb(&self) -> Box<dyn CBFn> {
        let db = self.db.clone();
        Box::new(move |relid, v, w| db.lock().unwrap().update(relid, v, w as isize))
    }
}

/// Single-threaded `UpdateHandler` implementation that stores updates
/// in a `DeltaMap`, locking the map for the entire duration of a commit.
/// After the commit is done, the map can be accessed from a different
/// thread.
#[derive(Clone, Debug)]
pub struct ValMapUpdateHandler {
    db: Arc<Mutex<DeltaMap<DDValue>>>,
    /// Stores pointer to `MutexGuard` between `before_commit()` and
    /// `after_commit()`.  This has to be unsafe, because Rust does
    /// not let us express a borrow from a field of the same struct in a
    /// safe way.
    locked: Arc<Cell<*mut libc::c_void>>,
}

impl Drop for ValMapUpdateHandler {
    /// Release the mutex if still held.
    fn drop<'a>(&'a mut self) {
        let guard_ptr =
            self.locked.replace(ptr::null_mut()) as *mut MutexGuard<'a, DeltaMap<DDValue>>;
        if !guard_ptr.is_null() {
            let _guard: Box<MutexGuard<'_, DeltaMap<DDValue>>> =
                unsafe { Box::from_raw(guard_ptr) };
        }
    }
}

impl ValMapUpdateHandler {
    pub fn new(db: Arc<Mutex<DeltaMap<DDValue>>>) -> Self {
        Self {
            db,
            locked: Arc::new(Cell::new(ptr::null_mut())),
        }
    }
}

impl UpdateHandler for ValMapUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let handler = self.clone();
        Box::new(move |relid, v, w| {
            let guard_ptr = handler.locked.get();
            // `update_cb` can also be called during rollback and stop operations.
            // Ignore those.
            if !guard_ptr.is_null() {
                let mut guard: Box<MutexGuard<'_, DeltaMap<DDValue>>> =
                    unsafe { Box::from_raw(guard_ptr as *mut MutexGuard<'_, DeltaMap<DDValue>>) };
                guard.update(relid, v, w);
                Box::into_raw(guard);
            }
        })
    }
    fn before_commit(&self) {
        let guard = Box::into_raw(Box::new(self.db.lock().unwrap())) as *mut libc::c_void;
        let old = self.locked.replace(guard);
        assert_eq!(old, ptr::null_mut());
    }
    fn after_commit(&self, _success: bool) {
        let guard_ptr = self.locked.replace(ptr::null_mut());
        assert_ne!(guard_ptr, ptr::null_mut());
        let _guard = unsafe { Box::from_raw(guard_ptr as *mut MutexGuard<'_, DeltaMap<DDValue>>) };
        // Lock will be released when `_guard` goes out of scope.
    }
}

/// `UpdateHandler` implementation that records _changes_ to output relations
/// rather than complete state.
#[derive(Clone, Debug)]
pub struct DeltaUpdateHandler {
    /// Setting the `DeltaMap` to `None` disables recording.
    db: Arc<Mutex<Option<DeltaMap<DDValue>>>>,
    locked: Arc<Cell<*mut libc::c_void>>,
}

impl Drop for DeltaUpdateHandler {
    /// Release the mutex if still held.
    fn drop<'a>(&'a mut self) {
        let guard_ptr =
            self.locked.replace(ptr::null_mut()) as *mut MutexGuard<'a, DeltaMap<DDValue>>;
        if !guard_ptr.is_null() {
            let _guard: Box<MutexGuard<'_, DeltaMap<DDValue>>> =
                unsafe { Box::from_raw(guard_ptr) };
        }
    }
}

impl DeltaUpdateHandler {
    pub fn new(db: Arc<Mutex<Option<DeltaMap<DDValue>>>>) -> Self {
        Self {
            db,
            locked: Arc::new(Cell::new(ptr::null_mut())),
        }
    }
}

impl UpdateHandler for DeltaUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let handler = self.clone();
        Box::new(move |relid, v, w| {
            let guard_ptr = handler.locked.get();
            if !guard_ptr.is_null() {
                let mut guard: Box<MutexGuard<'_, Option<DeltaMap<DDValue>>>> = unsafe {
                    Box::from_raw(guard_ptr as *mut MutexGuard<'_, Option<DeltaMap<DDValue>>>)
                };
                if let Some(db) = (*guard).as_mut() {
                    db.update(relid, v, w)
                };
                // make sure that guard does not get dropped
                Box::into_raw(guard);
            }
        })
    }
    fn before_commit(&self) {
        let guard = Box::into_raw(Box::new(self.db.lock().unwrap())) as *mut libc::c_void;
        let old = self.locked.replace(guard);
        assert_eq!(old, ptr::null_mut());
    }
    fn after_commit(&self, _success: bool) {
        let guard_ptr = self.locked.replace(ptr::null_mut());
        assert_ne!(guard_ptr, ptr::null_mut());
        let _guard = unsafe { Box::from_raw(guard_ptr as *mut MutexGuard<'_, DeltaMap<DDValue>>) };
        // Lock will be released when `_guard` goes out of scope.
    }
}

/// `UpdateHandler` implementation that chains multiple single-threaded
/// handlers.
#[derive(Debug)]
pub struct ChainedUpdateHandler {
    handlers: Vec<Box<dyn UpdateHandler>>,
}

impl ChainedUpdateHandler {
    pub fn new(handlers: Vec<Box<dyn UpdateHandler>>) -> Self {
        Self { handlers }
    }
}

impl UpdateHandler for ChainedUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let mut cbs: Vec<Box<dyn ST_CBFn>> = self.handlers.iter().map(|h| h.update_cb()).collect();
        Box::new(move |relid, v, w| {
            for cb in cbs.iter_mut() {
                cb(relid, v, w);
            }
        })
    }
    fn before_commit(&self) {
        for h in self.handlers.iter() {
            h.before_commit();
        }
    }

    fn after_commit(&self, success: bool) {
        for h in self.handlers.iter() {
            h.after_commit(success);
        }
    }
}

/// `UpdateHandler` implementation that chains multiple multi-threaded
/// handlers.
#[derive(Clone, Debug)]
pub struct MTChainedUpdateHandler {
    handlers: Vec<Box<dyn IMTUpdateHandler>>,
}

impl MTChainedUpdateHandler {
    pub fn new(handlers: Vec<Box<dyn IMTUpdateHandler>>) -> Self {
        Self { handlers }
    }
}

impl UpdateHandler for MTChainedUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let mut cbs: Vec<Box<dyn ST_CBFn>> = self.handlers.iter().map(|h| h.update_cb()).collect();
        Box::new(move |relid, v, w| {
            for cb in cbs.iter_mut() {
                cb(relid, v, w);
            }
        })
    }
    fn before_commit(&self) {
        for h in self.handlers.iter() {
            h.before_commit();
        }
    }

    fn after_commit(&self, success: bool) {
        for h in self.handlers.iter() {
            h.after_commit(success);
        }
    }
}

impl MTUpdateHandler for MTChainedUpdateHandler {
    fn mt_update_cb(&self) -> Box<dyn CBFn> {
        let mut cbs: Vec<Box<dyn CBFn>> = self.handlers.iter().map(|h| h.mt_update_cb()).collect();
        Box::new(move |relid, v, w| {
            for cb in cbs.iter_mut() {
                cb(relid, v, w);
            }
        })
    }
}

/// We use a single mpsc channel to notify worker about
/// update, start, and commit events.
enum Msg {
    BeforeCommit,
    Update { relid: RelId, v: DDValue, w: isize },
    AfterCommit { success: bool },
    Stop,
}

/// `UpdateHandler` implementation that handles updates in a separate
/// worker thread.
#[derive(Clone, Debug)]
pub struct ThreadUpdateHandler {
    /// Channel to worker thread.
    msg_channel: Arc<Mutex<Sender<Msg>>>,

    /// Barrier to synchronize completion of transaction with worker.
    commit_barrier: Arc<Barrier>,
}

impl ThreadUpdateHandler {
    pub fn new<F>(handler_generator: F) -> Self
    where
        F: FnOnce() -> Box<dyn UpdateHandler> + Send + 'static,
    {
        let (tx_msg_channel, rx_message_channel) = channel();
        let commit_barrier = Arc::new(Barrier::new(2));
        let commit_barrier2 = commit_barrier.clone();

        spawn(move || {
            let handler = handler_generator();
            let mut update_cb = handler.update_cb();
            loop {
                match rx_message_channel.recv() {
                    Ok(Msg::Update { relid, v, w }) => {
                        update_cb(relid, &v, w);
                    }
                    Ok(Msg::BeforeCommit) => handler.before_commit(),
                    Ok(Msg::AfterCommit { success }) => {
                        // All updates have been sent to channel by now: flush the channel.
                        loop {
                            match rx_message_channel.try_recv() {
                                Ok(Msg::Update { relid, v, w }) => {
                                    update_cb(relid, &v, w);
                                }
                                Ok(Msg::Stop) => return,
                                _ => break,
                            }
                        }

                        handler.after_commit(success);
                        commit_barrier2.wait();
                    }
                    Ok(Msg::Stop) => return,
                    _ => return,
                }
            }
        });

        Self {
            msg_channel: Arc::new(Mutex::new(tx_msg_channel)),
            commit_barrier,
        }
    }
}

impl Drop for ThreadUpdateHandler {
    fn drop(&mut self) {
        self.msg_channel.lock().unwrap().send(Msg::Stop).unwrap();
    }
}

impl UpdateHandler for ThreadUpdateHandler {
    fn update_cb(&self) -> Box<dyn ST_CBFn> {
        let channel = self.msg_channel.lock().unwrap().clone();
        Box::new(move |relid, v, w| {
            channel
                .send(Msg::Update {
                    relid,
                    v: v.clone(),
                    w,
                })
                .unwrap();
        })
    }

    fn before_commit(&self) {
        self.msg_channel
            .lock()
            .unwrap()
            .send(Msg::BeforeCommit)
            .unwrap();
    }

    fn after_commit(&self, success: bool) {
        if self
            .msg_channel
            .lock()
            .unwrap()
            .send(Msg::AfterCommit { success })
            .is_ok()
        {
            // Wait for all queued updates to get processed by worker.
            self.commit_barrier.wait();
        }
    }
}

impl MTUpdateHandler for ThreadUpdateHandler {
    fn mt_update_cb(&self) -> Box<dyn CBFn> {
        let channel = self.msg_channel.lock().unwrap().clone();
        Box::new(move |relid, v, w| {
            channel
                .send(Msg::Update {
                    relid,
                    v: v.clone(),
                    w: w as isize,
                })
                .unwrap();
        })
    }
}
