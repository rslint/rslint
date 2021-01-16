//! C Bindings
#![cfg(feature = "c_api")]

use crate::{
    api::HDDlog,
    record::{Record, UpdCmd},
    update_handler::{ExternCCallback, ExternCUpdateHandler, NullUpdateHandler},
};
use differential_datalog::{
    ddval::DDValue,
    program::{IdxId, RelId},
    record::IntoRecord,
    DDlog, DeltaMap,
};
use std::{
    collections::BTreeMap,
    ffi::{CStr, CString},
    fs::File,
    os::raw,
    ptr,
    sync::{Arc, Mutex},
};

#[cfg(feature = "flatbuf")]
use std::{mem, slice};

#[cfg(feature = "flatbuf")]
use super::flatbuf;
#[cfg(feature = "flatbuf")]
use super::flatbuf::FromFlatBuffer;

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, IntoRawHandle, RawHandle};

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_table_id(tname: *const raw::c_char) -> libc::size_t {
    if tname.is_null() {
        return libc::size_t::max_value();
    }

    let table_str = CStr::from_ptr(tname).to_str().unwrap();
    match HDDlog::get_table_id(table_str) {
        Ok(relid) => relid as libc::size_t,
        Err(_) => libc::size_t::max_value(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_table_name(tid: libc::size_t) -> *const raw::c_char {
    match HDDlog::get_table_cname(tid) {
        Ok(name) => name.as_ptr(),
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_index_id(iname: *const raw::c_char) -> libc::size_t {
    if iname.is_null() {
        return libc::size_t::max_value();
    }

    let index_str = CStr::from_ptr(iname).to_str().unwrap();
    match HDDlog::get_index_id(index_str) {
        Ok(idxid) => idxid as libc::size_t,
        Err(_) => libc::size_t::max_value(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_index_name(iid: libc::size_t) -> *const raw::c_char {
    match HDDlog::get_index_cname(iid) {
        Ok(name) => name.as_ptr(),
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_run(
    workers: raw::c_uint,
    do_store: bool,
    print_err: Option<extern "C" fn(msg: *const raw::c_char)>,
    init_state: *mut *mut DeltaMap<DDValue>,
) -> *const HDDlog {
    let result = HDDlog::do_run(workers as usize, do_store, print_err);

    match result {
        Ok((hddlog, init)) => {
            if !init_state.is_null() {
                *init_state = Box::into_raw(Box::new(init));
            };
            Arc::into_raw(Arc::new(hddlog))
        }
        Err(err) => {
            HDDlog::print_err(print_err, &format!("ddlog_run() failed: {}", err));
            ptr::null()
        }
    }
}

#[no_mangle]
#[cfg(unix)]
pub unsafe extern "C" fn ddlog_record_commands(prog: *const HDDlog, fd: RawFd) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let mut prog = Arc::from_raw(prog);

    let file = if fd == -1 {
        None
    } else {
        Some(File::from_raw_fd(fd))
    };

    let res = match Arc::get_mut(&mut prog) {
        Some(prog) => {
            let mut old_file = file.map(Mutex::new);
            prog.record_commands(&mut old_file);
            /* Convert the old file into FD to prevent it from closing.
             * It is the caller's responsibility to close the file when
             * they are done with it. */
            old_file.map(|m| m.into_inner().unwrap().into_raw_fd());
            0
        }
        None => -1,
    };

    Arc::into_raw(prog);
    res
}

#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn ddlog_record_commands(prog: *const HDDlog, fd: raw::c_int) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let mut prog = Arc::from_raw(prog);

    let file = if fd == -1 {
        None
    } else {
        // Convert file descriptor to file handle on Windows.
        let handle = libc::get_osfhandle(fd);
        Some(File::from_raw_handle(handle as RawHandle))
    };

    let res = match Arc::get_mut(&mut prog) {
        Some(prog) => {
            let mut old_file = file.map(Mutex::new);
            prog.record_commands(&mut old_file);
            /* Convert the old file into FD to prevent it from closing.
             * It is the caller's responsibility to close the file when
             * they are done with it. */
            old_file.map(|m| m.into_inner().unwrap().into_raw_handle());
            0
        }
        None => -1,
    };

    Arc::into_raw(prog);
    res
}

#[no_mangle]
#[cfg(unix)]
pub unsafe extern "C" fn ddlog_dump_input_snapshot(prog: *const HDDlog, fd: RawFd) -> raw::c_int {
    if prog.is_null() || fd < 0 {
        return -1;
    }

    let prog = Arc::from_raw(prog);
    let mut file = File::from_raw_fd(fd);
    let res = prog
        .dump_input_snapshot(&mut file)
        .map(|_| 0)
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_dump_input_snapshot: error: {}", e));
            -1
        });

    file.into_raw_fd();
    Arc::into_raw(prog);
    res
}

#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn ddlog_dump_input_snapshot(
    prog: *const HDDlog,
    fd: raw::c_int,
) -> raw::c_int {
    if prog.is_null() || fd < 0 {
        return -1;
    }

    let prog = &*prog;
    // Convert file descriptor to file handle on Windows.
    let handle = libc::get_osfhandle(fd);
    let mut file = File::from_raw_handle(handle as RawHandle);
    let res = prog
        .dump_input_snapshot(&mut file)
        .map(|_| 0)
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_dump_input_snapshot: error: {}", e));
            -1
        });

    file.into_raw_handle();
    res
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_stop(prog: *const HDDlog) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    /* Prevents closing of the old descriptor. */
    ddlog_record_commands(prog, -1);

    let prog = Arc::from_raw(prog);
    match Arc::try_unwrap(prog) {
        Ok(HDDlog {
            prog, print_err, ..
        }) => prog
            .into_inner()
            .map(|mut p| {
                p.stop().map(|_| 0).unwrap_or_else(|e| {
                    HDDlog::print_err(print_err, &format!("ddlog_stop(): error: {}", e));
                    -1
                })
            })
            .unwrap_or_else(|e| {
                HDDlog::print_err(
                    print_err,
                    &format!("ddlog_stop(): error acquiring lock: {}", e),
                );
                -1
            }),
        Err(pref) => {
            pref.eprintln("ddlog_stop(): cannot extract value from Arc");
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_start(prog: *const HDDlog) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.transaction_start().map(|_| 0).unwrap_or_else(|e| {
        prog.eprintln(&format!("ddlog_transaction_start(): error: {}", e));
        -1
    })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_commit_dump_changes(
    prog: *const HDDlog,
) -> *mut DeltaMap<DDValue> {
    if prog.is_null() {
        return ptr::null_mut();
    }
    let prog = &*prog;

    prog.transaction_commit_dump_changes()
        .map(|delta| Box::into_raw(Box::new(delta)))
        .unwrap_or_else(|e| {
            prog.eprintln(&format!(
                "ddlog_transaction_commit_dump_changes: error: {}",
                e
            ));
            ptr::null_mut()
        })
}

#[repr(C)]
pub struct ddlog_record_update {
    table: libc::size_t,
    rec: *mut Record,
    w: libc::ssize_t,
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_commit_dump_changes_as_array(
    prog: *const HDDlog,
    changes: *mut *const ddlog_record_update,
    num_changes: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    do_transaction_commit_dump_changes_as_array(prog, changes, num_changes)
        .map(|_| 0)
        .unwrap_or_else(|e| {
            prog.eprintln(&format!(
                "ddlog_transaction_commit_dump_changes_as_array: error: {}",
                e
            ));
            -1
        })
}

unsafe fn do_transaction_commit_dump_changes_as_array(
    prog: &HDDlog,
    changes: *mut *const ddlog_record_update,
    num_changes: *mut libc::size_t,
) -> Result<(), String> {
    let updates = prog.transaction_commit_dump_changes()?;
    let mut size = 0;
    for (_, delta) in updates.as_ref().iter() {
        size += delta.len();
    }

    *num_changes = size;
    // Make sure that vector's capacity will be equal to its length.
    let mut change_vec = Vec::with_capacity(size);
    for (rel, delta) in updates.into_iter() {
        for (val, w) in delta.into_iter() {
            change_vec.push(ddlog_record_update {
                table: rel,
                rec: Box::into_raw(Box::new(val.into_record())),
                w,
            });
        }
    }

    *changes = change_vec.as_ptr();
    std::mem::forget(change_vec);

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_free_record_updates(
    changes: *mut ddlog_record_update,
    num_changes: libc::size_t,
) {
    // Assume that vector's capacity is equal to its length.
    let changes_vec: Vec<ddlog_record_update> =
        Vec::from_raw_parts(changes, num_changes as usize, num_changes as usize);
    for upd in changes_vec.into_iter() {
        let upd: Box<Record> = Box::from_raw(upd.rec);
    }
}

#[cfg(feature = "flatbuf")]
#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_commit_dump_changes_to_flatbuf(
    prog: *const HDDlog,
    buf: *mut *const u8,
    buf_size: *mut libc::size_t,
    buf_capacity: *mut libc::size_t,
    buf_offset: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null() || buf_size.is_null() || buf_capacity.is_null() || buf_offset.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.transaction_commit_dump_changes()
        .map(|changes| {
            let (fbvec, fboffset) = flatbuf::updates_to_flatbuf(&changes);
            *buf = fbvec.as_ptr();
            *buf_size = fbvec.len() as libc::size_t;
            *buf_capacity = fbvec.capacity() as libc::size_t;
            *buf_offset = fboffset as libc::size_t;
            mem::forget(fbvec);
            0
        })
        .unwrap_or_else(|e| {
            prog.eprintln(&format!(
                "ddlog_transaction_commit_dump_changes_to_flatbuf: error: {}",
                e
            ));
            -1
        })
}

#[cfg(not(feature = "flatbuf"))]
#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_commit_dump_changes_to_flatbuf(
    prog: *const HDDlog,
    _buf: *mut *const u8,
    _buf_size: *mut libc::size_t,
    _buf_capacity: *mut libc::size_t,
    _buf_offset: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }

    let prog = &*prog;
    prog.eprintln("ddlog_transaction_commit_dump_changes_to_flatbuf(): error: DDlog was compiled without FlatBuffers support");

    -1
}

#[cfg(feature = "flatbuf")]
#[no_mangle]
pub unsafe extern "C" fn ddlog_query_index_from_flatbuf(
    prog: *const HDDlog,
    buf: *const u8,
    n: libc::size_t,
    resbuf: *mut *const u8,
    resbuf_size: *mut libc::size_t,
    resbuf_capacity: *mut libc::size_t,
    resbuf_offset: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null()
        || buf.is_null()
        || resbuf.is_null()
        || resbuf_size.is_null()
        || resbuf_capacity.is_null()
        || resbuf_offset.is_null()
    {
        return -1;
    }

    let prog = &*prog;

    flatbuf::query_from_flatbuf(slice::from_raw_parts(buf, n))
        .and_then(|(idxid, key)| {
            prog.query_index(idxid, key).map(|res| {
                let (fbvec, fboffset) = flatbuf::idx_values_to_flatbuf(idxid, res.iter());
                *resbuf = fbvec.as_ptr();
                *resbuf_size = fbvec.len() as libc::size_t;
                *resbuf_capacity = fbvec.capacity() as libc::size_t;
                *resbuf_offset = fboffset as libc::size_t;
                mem::forget(fbvec);
                0
            })
        })
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_query_index_from_flatbuf(): error: {}", e));
            -1
        })
}

#[cfg(not(feature = "flatbuf"))]
#[no_mangle]
pub unsafe extern "C" fn ddlog_query_index_from_flatbuf(
    prog: *const HDDlog,
    _buf: *const u8,
    _n: libc::size_t,
    _resbuf: *mut *const u8,
    _resbuf_size: *mut libc::size_t,
    _resbuf_capacity: *mut libc::size_t,
    _resbuf_offset: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }

    let prog = &*prog;
    prog.eprintln(
        "ddlog_query_index_from_flatbuf(): error: DDlog was compiled without FlatBuffers support",
    );

    -1
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_dump_index(
    prog: *const HDDlog,
    idxid: libc::size_t,
    cb: Option<extern "C" fn(arg: libc::uintptr_t, rec: *const Record)>,
    cb_arg: libc::uintptr_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.dump_index(idxid as IdxId)
        .map(|set| {
            if let Some(f) = cb {
                for val in set.iter() {
                    f(cb_arg, &val.clone().into_record());
                }
            };
            0
        })
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_dump_index: error: {}", e));
            -1
        })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_query_index(
    prog: *const HDDlog,
    idxid: libc::size_t,
    key: *const Record,
    cb: Option<extern "C" fn(arg: libc::uintptr_t, rec: *const Record)>,
    cb_arg: libc::uintptr_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.query_index_rec(idxid as IdxId, &*key)
        .map(|set| {
            if let Some(f) = cb {
                for val in set.iter() {
                    f(cb_arg, &val.clone().into_record());
                }
            }
            0
        })
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_dump_index: error: {}", e));
            -1
        })
}

#[cfg(feature = "flatbuf")]
#[no_mangle]
pub unsafe extern "C" fn ddlog_dump_index_to_flatbuf(
    prog: *const HDDlog,
    idxid: libc::size_t,
    resbuf: *mut *const u8,
    resbuf_size: *mut libc::size_t,
    resbuf_capacity: *mut libc::size_t,
    resbuf_offset: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null()
        || resbuf.is_null()
        || resbuf_size.is_null()
        || resbuf_capacity.is_null()
        || resbuf_offset.is_null()
    {
        return -1;
    }
    let prog = &*prog;

    prog.dump_index(idxid as IdxId)
        .map(|res| {
            let (fbvec, fboffset) = flatbuf::idx_values_to_flatbuf(idxid, res.iter());
            *resbuf = fbvec.as_ptr();
            *resbuf_size = fbvec.len() as libc::size_t;
            *resbuf_capacity = fbvec.capacity() as libc::size_t;
            *resbuf_offset = fboffset as libc::size_t;
            mem::forget(fbvec);
            0
        })
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_dump_index_to_flatbuf(): error: {}", e));
            -1
        })
}

#[cfg(not(feature = "flatbuf"))]
#[no_mangle]
pub unsafe extern "C" fn ddlog_dump_index_to_flatbuf(
    prog: *const HDDlog,
    _idxid: libc::size_t,
    _resbuf: *mut *const u8,
    _resbuf_size: *mut libc::size_t,
    _resbuf_capacity: *mut libc::size_t,
    _resbuf_offset: *mut libc::size_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }

    let prog = &*prog;
    prog.eprintln(
        "ddlog_dump_index_to_flatbuf(): error: DDlog was compiled without FlatBuffers support",
    );

    -1
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_flatbuf_free(
    buf: *mut u8,
    buf_size: libc::size_t,
    buf_capacity: libc::size_t,
) {
    Vec::from_raw_parts(buf, buf_size as usize, buf_capacity as usize);
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_commit(prog: *const HDDlog) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.transaction_commit().map(|_| 0).unwrap_or_else(|e| {
        prog.eprintln(&format!("ddlog_transaction_commit(): error: {}", e));
        -1
    })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_transaction_rollback(prog: *const HDDlog) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.transaction_rollback().map(|_| 0).unwrap_or_else(|e| {
        prog.eprintln(&format!("ddlog_transaction_rollback(): error: {}", e));
        -1
    })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_apply_updates(
    prog: *const HDDlog,
    upds: *const *mut UpdCmd,
    n: libc::size_t,
) -> raw::c_int {
    if prog.is_null() || upds.is_null() {
        return -1;
    }
    let prog = Arc::from_raw(prog);

    let res = prog
        .apply_updates((0..n).map(|i| Box::from_raw(*upds.add(i))))
        .map(|_| 0)
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_apply_updates(): error: {}", e));
            -1
        });
    Arc::into_raw(prog);
    res
}

#[cfg(feature = "flatbuf")]
#[no_mangle]
pub unsafe extern "C" fn ddlog_apply_updates_from_flatbuf(
    prog: *const HDDlog,
    buf: *const u8,
    n: libc::size_t,
) -> raw::c_int {
    if prog.is_null() || buf.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.apply_updates_from_flatbuf(slice::from_raw_parts(buf, n))
        .map(|_| 0)
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("ddlog_apply_updates_from_flatbuf(): error: {}", e));
            -1
        })
}

#[cfg(not(feature = "flatbuf"))]
#[no_mangle]
pub unsafe extern "C" fn ddlog_apply_updates_from_flatbuf(
    prog: *const HDDlog,
    _buf: *const u8,
    _n: libc::size_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }

    let prog = &*prog;
    prog.eprintln(&"ddlog_apply_updates_from_flatbuf(): error: DDlog was compiled without FlatBuffers support");

    -1
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_clear_relation(
    prog: *const HDDlog,
    table: libc::size_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.clear_relation(table).map(|_| 0).unwrap_or_else(|e| {
        prog.eprintln(&format!("ddlog_clear_relation(): error: {}", e));
        -1
    })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_dump_table(
    prog: *const HDDlog,
    table: libc::size_t,
    cb: Option<extern "C" fn(arg: libc::uintptr_t, rec: *const Record, w: libc::ssize_t) -> bool>,
    cb_arg: libc::uintptr_t,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    let f = cb.map(|f| move |rec: &Record, w: isize| f(cb_arg, rec, w as libc::ssize_t));

    prog.dump_table(table, f).map(|_| 0).unwrap_or_else(|e| {
        prog.eprintln(&format!("ddlog_dump_table(): error: {}", e));
        -1
    })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_enable_cpu_profiling(
    prog: *const HDDlog,
    enable: bool,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.enable_cpu_profiling(enable);
    0
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_enable_timely_profiling(
    prog: *const HDDlog,
    enable: bool,
) -> raw::c_int {
    if prog.is_null() {
        return -1;
    }
    let prog = &*prog;

    prog.enable_timely_profiling(enable);
    0
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_profile(prog: *const HDDlog) -> *const raw::c_char {
    if prog.is_null() {
        return ptr::null();
    }
    let prog = &*prog;

    let profile = prog.profile();
    CString::new(profile)
        .map(CString::into_raw)
        .unwrap_or_else(|e| {
            prog.eprintln(&format!("Failed to convert profile string to C: {}", e));
            ptr::null_mut()
        })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_string_free(s: *mut raw::c_char) {
    if s.is_null() {
        return;
    }

    CString::from_raw(s);
}

#[no_mangle]
pub extern "C" fn ddlog_new_delta() -> *mut DeltaMap<DDValue> {
    Box::into_raw(Box::new(DeltaMap::new()))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delta_get_table(
    delta: *const DeltaMap<DDValue>,
    table: libc::size_t,
) -> *mut DeltaMap<DDValue> {
    let res = DeltaMap::singleton(
        table,
        (&*delta)
            .try_get_rel(table as RelId)
            .cloned()
            .unwrap_or_else(BTreeMap::new),
    );

    Box::into_raw(Box::new(res))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delta_enumerate(
    delta: *const DeltaMap<DDValue>,
    cb: Option<ExternCCallback>,
    cb_arg: libc::uintptr_t,
) {
    if let Some(f) = cb {
        for (table_id, table_data) in (&*delta).as_ref().iter() {
            for (val, weight) in table_data.iter() {
                assert!(*weight == 1 || *weight == -1);
                f(
                    cb_arg,
                    *table_id as libc::size_t,
                    &val.clone().into_record(),
                    *weight as libc::ssize_t,
                );
            }
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delta_clear_table(
    delta: *mut DeltaMap<DDValue>,
    table: libc::size_t,
) {
    (&mut *delta).clear_rel(table as RelId);
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delta_remove_table(
    delta: *mut DeltaMap<DDValue>,
    table: libc::size_t,
) -> *mut DeltaMap<DDValue> {
    Box::into_raw(Box::new(DeltaMap::singleton(
        table,
        (&mut *delta).clear_rel(table as RelId),
    )))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delta_clear(delta: *mut DeltaMap<DDValue>) {
    (&mut *delta).as_mut().clear();
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delta_union(
    delta: *mut DeltaMap<DDValue>,
    new_delta: *const DeltaMap<DDValue>,
) {
    for (table_id, table_data) in (&*new_delta).as_ref().iter() {
        for (val, weight) in table_data.iter() {
            (&mut *delta).update(*table_id, val, *weight);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_free_delta(delta: *mut DeltaMap<DDValue>) {
    if !delta.is_null() {
        // Deallocate the DeltaMap
        Box::from_raw(delta);
    }
}
