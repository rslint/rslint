//! C API to Record and UpdCmd
#![cfg(feature = "c_api")]

use crate::record::{CollectionKind, Name, Record, RelIdentifier, UpdCmd};
use num::{BigInt, ToPrimitive};
use ordered_float::OrderedFloat;
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    ptr, slice, str,
};

#[no_mangle]
pub unsafe extern "C" fn ddlog_dump_record(record: *const Record) -> *mut libc::c_char {
    record
        .as_ref()
        .and_then(|record| CString::new(record.to_string()).ok())
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_free(rec: *mut Record) {
    Box::from_raw(rec);
}

#[no_mangle]
pub extern "C" fn ddlog_bool(b: bool) -> *mut Record {
    Box::into_raw(Box::new(Record::Bool(b)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_bool(rec: *const Record) -> bool {
    rec.as_ref().map(Record::is_bool).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_bool(rec: *const Record) -> bool {
    rec.as_ref().and_then(Record::as_bool).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_int(rec: *const Record) -> bool {
    rec.as_ref().map(Record::is_int).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_float(rec: *const Record) -> bool {
    rec.as_ref().map(Record::is_float).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn ddlog_float(float: f32) -> *mut Record {
    Box::into_raw(Box::new(Record::Float(OrderedFloat(float))))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_float(rec: *const Record) -> f32 {
    rec.as_ref()
        .and_then(Record::as_float)
        .map(|float| *float)
        .unwrap_or(0.0)
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_double(rec: *const Record) -> bool {
    rec.as_ref().map(Record::is_double).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn ddlog_double(v: f64) -> *mut Record {
    Box::into_raw(Box::new(Record::Double(OrderedFloat(v))))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_double(rec: *const Record) -> f64 {
    rec.as_ref()
        .and_then(Record::as_double)
        .map(|float| *float)
        .unwrap_or(0.0)
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_int(int: *const libc::c_uchar, size: libc::size_t) -> *mut Record {
    Box::into_raw(Box::new(Record::Int(BigInt::from_signed_bytes_be(
        slice::from_raw_parts(int as *const u8, size as usize),
    ))))
}

/// `buf`        - buffer to store the big-endian byte representation of the integer value
/// `capacity`   - buffer capacity
///
/// Return value: if `capacity` is 0, returns the minimal buffer capacity necessary to
/// represent the value otherwise returns the number of bytes stored in `buf` or `-1` if `buf`
/// is not big enough.
#[no_mangle]
pub unsafe extern "C" fn ddlog_get_int(
    rec: *const Record,
    buf: *mut libc::c_uchar,
    capacity: libc::size_t,
) -> libc::ssize_t {
    match rec.as_ref() {
        Some(Record::Int(i)) => {
            let bytes = i.to_signed_bytes_be();

            if capacity == 0 {
                bytes.len() as libc::ssize_t
            } else if capacity >= bytes.len() {
                for (i, b) in bytes.iter().enumerate() {
                    if let Some(p) = buf.add(i).as_mut() {
                        *p = *b;
                    }
                }

                bytes.len() as libc::ssize_t
            } else {
                -1
            }
        }
        _ => 0,
    }
}

/// Determines the fewest bits necessary to express the integer value, not including the sign.
#[no_mangle]
pub unsafe extern "C" fn ddlog_int_bits(rec: *const Record) -> libc::size_t {
    match rec.as_ref() {
        Some(Record::Int(bigint)) => bigint.bits() as libc::size_t,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn ddlog_u64(v: u64) -> *mut Record {
    Box::into_raw(Box::new(Record::Int(BigInt::from(v))))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_u64(rec: *const Record) -> u64 {
    rec.as_ref()
        .and_then(Record::as_int)
        .and_then(BigInt::to_u64)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ddlog_i64(v: i64) -> *mut Record {
    Box::into_raw(Box::new(Record::Int(BigInt::from(v))))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_i64(rec: *const Record) -> i64 {
    rec.as_ref()
        .and_then(Record::as_int)
        .and_then(BigInt::to_i64)
        .unwrap_or(0)
}

// FIXME: 128 bit integers are not FFI-safe, so we need to find an alternate
//        method to make this defined behavior
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn ddlog_u128(v: u128) -> *mut Record {
    Box::into_raw(Box::new(Record::Int(BigInt::from(v))))
}

// FIXME: 128 bit integers are not FFI-safe, so we need to find an alternate
//        method to make this defined behavior
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn ddlog_get_u128(rec: *const Record) -> u128 {
    rec.as_ref()
        .and_then(Record::as_int)
        .and_then(BigInt::to_u128)
        .unwrap_or(0)
}

// FIXME: 128 bit integers are not FFI-safe, so we need to find an alternate
//        method to make this defined behavior
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn ddlog_i128(v: i128) -> *mut Record {
    Box::into_raw(Box::new(Record::Int(BigInt::from(v))))
}

// FIXME: 128 bit integers are not FFI-safe, so we need to find an alternate
//        method to make this defined behavior
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn ddlog_get_i128(rec: *const Record) -> i128 {
    rec.as_ref()
        .and_then(Record::as_int)
        .and_then(BigInt::to_i128)
        .unwrap_or(0)
}

/// Returns NULL if the given string is not valid UTF8
#[no_mangle]
pub unsafe extern "C" fn ddlog_string(string: *const libc::c_char) -> *mut Record {
    if let Ok(string) = CStr::from_ptr(string).to_str() {
        Box::into_raw(Box::new(Record::String(string.to_owned())))
    } else {
        ptr::null_mut()
    }
}

/// Returns NULL if s is not a valid UTF8 string.
#[no_mangle]
pub unsafe extern "C" fn ddlog_string_with_length(
    s: *const libc::c_char,
    len: libc::size_t,
) -> *mut Record {
    // If `len` is zero, return empty string even if `s` is `NULL`.
    if len == 0 {
        return Box::into_raw(Box::new(Record::String("".to_owned())));
    }

    if s.is_null() {
        return ptr::null_mut();
    }

    if let Ok(string) = str::from_utf8(slice::from_raw_parts(s as *const u8, len as usize)) {
        Box::into_raw(Box::new(Record::String(string.to_owned())))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_string(rec: *const Record) -> bool {
    match rec.as_ref() {
        Some(Record::String(_)) => true,
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_strlen(rec: *const Record) -> libc::size_t {
    match rec.as_ref() {
        Some(Record::String(s)) => s.len() as libc::size_t,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_str_with_length(
    rec: *const Record,
    len: *mut libc::size_t,
) -> *const libc::c_char {
    match rec.as_ref() {
        Some(Record::String(s)) => {
            *len = s.len() as libc::size_t;
            s.as_ptr() as *const libc::c_char
        }
        _ => ptr::null(),
    }
}

/// Returns NULL if s is not a valid UTF8 string.
#[no_mangle]
pub unsafe extern "C" fn ddlog_serialized(
    t: *const libc::c_char,
    s: *const libc::c_char,
) -> *mut Record {
    let t = match CStr::from_ptr(t).to_str() {
        Ok(t) => t,
        Err(_) => return ptr::null_mut(),
    };

    if let Ok(string) = CStr::from_ptr(s).to_str() {
        Box::into_raw(Box::new(Record::Serialized(
            Cow::from(t),
            string.to_owned(),
        )))
    } else {
        ptr::null_mut()
    }
}

/// Returns NULL if s is not a valid UTF8 string.
#[no_mangle]
pub unsafe extern "C" fn ddlog_serialized_with_length(
    t: *const libc::c_char,
    t_len: libc::size_t,
    s: *const libc::c_char,
    s_len: libc::size_t,
) -> *mut Record {
    let t = match str::from_utf8(slice::from_raw_parts(t as *const u8, t_len as usize)) {
        Ok(str) => str,
        Err(_) => return ptr::null_mut(),
    };

    // If `s_len` is zero, return empty string even if `s` is `NULL`.
    if s_len == 0 {
        return Box::into_raw(Box::new(Record::Serialized(Cow::from(t), "".to_owned())));
    }

    if s.is_null() {
        return ptr::null_mut();
    }

    if let Ok(string) = str::from_utf8(slice::from_raw_parts(s as *const u8, s_len as usize)) {
        Box::into_raw(Box::new(Record::Serialized(
            Cow::from(t),
            string.to_owned(),
        )))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_serialized(rec: *const Record) -> bool {
    matches!(rec.as_ref(), Some(Record::Serialized(_, _)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_tuple(fields: *const *mut Record, len: libc::size_t) -> *mut Record {
    let fields = ffi_record_vec(fields, len);
    Box::into_raw(Box::new(Record::Tuple(fields)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_tuple(rec: *const Record) -> bool {
    matches!(rec.as_ref(), Some(Record::Tuple(_)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_tuple_size(rec: *const Record) -> libc::size_t {
    match rec.as_ref() {
        Some(Record::Tuple(recs)) => recs.len() as libc::size_t,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_tuple_field(
    rec: *const Record,
    idx: libc::size_t,
) -> *const Record {
    rec.as_ref()
        .and_then(Record::as_tuple)
        .and_then(|records| records.get(idx))
        .map(|record| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

/// Convenience method to create a 2-tuple.
#[no_mangle]
pub unsafe extern "C" fn ddlog_pair(v1: *mut Record, v2: *mut Record) -> *mut Record {
    let v1 = Box::from_raw(v1);
    let v2 = Box::from_raw(v2);
    Box::into_raw(Box::new(Record::Tuple(vec![*v1, *v2])))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_tuple_push(tup: *mut Record, rec: *mut Record) {
    let rec = Box::from_raw(rec);
    let mut tup = Box::from_raw(tup);

    if let Record::Tuple(recs) = tup.as_mut() {
        recs.push(*rec)
    }

    Box::into_raw(tup);
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_vector(
    fields: *const *mut Record,
    len: libc::size_t,
) -> *mut Record {
    let fields = ffi_record_vec(fields, len);
    Box::into_raw(Box::new(Record::Array(CollectionKind::Vector, fields)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_vector(rec: *const Record) -> bool {
    match rec.as_ref() {
        Some(Record::Array(CollectionKind::Vector, _)) => true,
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_vector_size(rec: *const Record) -> libc::size_t {
    match rec.as_ref() {
        Some(Record::Array(CollectionKind::Vector, recs)) => recs.len() as libc::size_t,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_vector_elem(
    rec: *const Record,
    idx: libc::size_t,
) -> *const Record {
    rec.as_ref()
        .and_then(Record::as_vector)
        .and_then(|records| records.get(idx))
        .map(|record| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_vector_push(vec: *mut Record, rec: *mut Record) {
    let rec = Box::from_raw(rec);
    let mut vec = Box::from_raw(vec);

    if let Record::Array(CollectionKind::Vector, recs) = vec.as_mut() {
        recs.push(*rec)
    }

    Box::into_raw(vec);
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_set(fields: *const *mut Record, len: libc::size_t) -> *mut Record {
    let fields = ffi_record_vec(fields, len);
    Box::into_raw(Box::new(Record::Array(CollectionKind::Set, fields)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_set(rec: *const Record) -> bool {
    matches!(rec.as_ref(), Some(Record::Array(CollectionKind::Set, _)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_set_size(rec: *const Record) -> libc::size_t {
    rec.as_ref()
        .and_then(Record::as_set)
        .map(|set| set.len())
        .unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_set_elem(
    rec: *const Record,
    idx: libc::size_t,
) -> *const Record {
    rec.as_ref()
        .and_then(Record::as_set)
        .and_then(|records| records.get(idx))
        .map(|record| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_set_push(set: *mut Record, rec: *mut Record) {
    let rec = Box::from_raw(rec);
    let mut set = Box::from_raw(set);

    if let Record::Array(CollectionKind::Set, recs) = set.as_mut() {
        recs.push(*rec)
    }

    Box::into_raw(set);
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_map(fields: *const *mut Record, len: libc::size_t) -> *mut Record {
    let fields = ffi_record_vec(fields, len);
    Box::into_raw(Box::new(Record::Array(CollectionKind::Map, fields)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_map(rec: *const Record) -> bool {
    match rec.as_ref() {
        Some(Record::Array(CollectionKind::Map, _)) => true,
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_map_size(rec: *const Record) -> libc::size_t {
    match rec.as_ref() {
        Some(Record::Array(CollectionKind::Map, recs)) => recs.len() as libc::size_t,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_map_key(rec: *const Record, idx: libc::size_t) -> *const Record {
    rec.as_ref()
        .and_then(Record::as_map)
        .and_then(|records| records.get(idx))
        .and_then(Record::as_tuple)
        .and_then(|tuple| if tuple.len() == 2 { tuple.get(0) } else { None })
        .map(|record| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_map_val(rec: *const Record, idx: libc::size_t) -> *const Record {
    rec.as_ref()
        .and_then(Record::as_map)
        .and_then(|records| records.get(idx))
        .and_then(Record::as_tuple)
        .and_then(|tuple| if tuple.len() == 2 { tuple.get(1) } else { None })
        .map(|record| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_map_push(map: *mut Record, key: *mut Record, val: *mut Record) {
    let tup = Record::Tuple(vec![*Box::from_raw(key), *Box::from_raw(val)]);
    let mut map = Box::from_raw(map);

    if let Record::Array(CollectionKind::Map, recs) = map.as_mut() {
        recs.push(tup)
    }

    Box::into_raw(map);
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_struct(
    constructor: *const libc::c_char,
    fields: *const *mut Record,
    len: libc::size_t,
) -> *mut Record {
    let fields = ffi_record_vec(fields, len);

    if let Ok(constructor) = CStr::from_ptr(constructor).to_str() {
        Box::into_raw(Box::new(Record::PosStruct(
            Cow::from(constructor.to_owned()),
            fields,
        )))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_named_struct(
    constructor: *const libc::c_char,
    field_names: *const *const libc::c_char,
    fields: *const *mut Record,
    len: libc::size_t,
) -> *mut Record {
    let constructor = match CStr::from_ptr(constructor).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let names: &[*const libc::c_char] = slice::from_raw_parts(field_names, len);
    let mut tuples: Vec<(Name, Record)> = Vec::with_capacity(len as usize);

    for (index, n) in names.iter().enumerate() {
        let name = match CStr::from_ptr(*n).to_str() {
            Ok(s) => s,
            _ => return ptr::null_mut(),
        };

        let record = Box::from_raw(*fields.add(index));
        let tuple = (Cow::from(name.to_owned()), *record);

        tuples.push(tuple)
    }

    Box::into_raw(Box::new(Record::NamedStruct(
        Cow::from(constructor.to_owned()),
        tuples,
    )))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_struct_with_length(
    constructor: *const libc::c_char,
    constructor_len: libc::size_t,
    fields: *const *mut Record,
    len: libc::size_t,
) -> *mut Record {
    if constructor.is_null() {
        return ptr::null_mut();
    }

    let fields = ffi_record_vec(fields, len);
    let constructor = str::from_utf8(slice::from_raw_parts(
        constructor as *const u8,
        constructor_len as usize,
    ));

    if let Ok(constructor) = constructor {
        Box::into_raw(Box::new(Record::PosStruct(
            Cow::from(constructor.to_owned()),
            fields,
        )))
    } else {
        ptr::null_mut()
    }
}

/// Similar to `ddlog_struct()`, but expects `constructor` to be static string.
/// Doesn't allocate memory for a local copy of the string.
#[no_mangle]
pub unsafe extern "C" fn ddlog_struct_static_cons(
    constructor: *const libc::c_char,
    fields: *const *mut Record,
    len: libc::size_t,
) -> *mut Record {
    let fields = ffi_record_vec(fields, len);

    if let Ok(constructor) = CStr::from_ptr(constructor).to_str() {
        Box::into_raw(Box::new(Record::PosStruct(Cow::from(constructor), fields)))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_struct_static_cons_with_length(
    constructor: *const libc::c_char,
    constructor_len: libc::size_t,
    fields: *const *mut Record,
    len: libc::size_t,
) -> *mut Record {
    if constructor.is_null() {
        return ptr::null_mut();
    }

    let fields = ffi_record_vec(fields, len);
    let constructor = str::from_utf8(slice::from_raw_parts(
        constructor as *const u8,
        constructor_len as usize,
    ));

    if let Ok(constructor) = constructor {
        Box::into_raw(Box::new(Record::PosStruct(Cow::from(constructor), fields)))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_struct(rec: *const Record) -> bool {
    rec.as_ref().map(Record::is_struct).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_is_named_struct(rec: *const Record) -> bool {
    rec.as_ref()
        .map(Record::is_named_struct)
        .unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_struct_field(
    rec: *const Record,
    idx: libc::size_t,
) -> *const Record {
    rec.as_ref()
        .and_then(|record| record.nth_struct_field(idx as usize))
        .map(|record| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_named_struct_field(
    rec: *const Record,
    name: *const libc::c_char,
) -> *const Record {
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        _ => return ptr::null_mut(),
    };

    rec.as_ref()
        .and_then(Record::named_struct_fields)
        .and_then(|fields| fields.iter().find(|(field, _)| field == name))
        .map(|(_, record)| record as *const Record)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_named_struct_field_name(
    rec: *const Record,
    idx: libc::size_t,
    len: *mut libc::size_t,
) -> *const libc::c_char {
    rec.as_ref()
        .and_then(Record::named_struct_fields)
        .and_then(|fields| fields.get(idx))
        .map(|field| {
            // Set the out length to the length of the field's name
            *len = field.0.len();
            field.0.as_ref().as_ptr() as *const libc::c_char
        })
        .unwrap_or_else(|| {
            // If this wasn't a named struct or the index was incorrect,
            // set the out length to zero
            *len = 0;
            ptr::null()
        })
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_get_constructor_with_length(
    rec: *const Record,
    len: *mut libc::size_t,
) -> *const libc::c_char {
    rec.as_ref()
        .and_then(Record::struct_constructor)
        .map(|constructor| {
            // Set the out length to the length of the constructor's name
            *len = constructor.len();
            constructor.as_ref().as_ptr() as *const libc::c_char
        })
        .unwrap_or_else(|| {
            // If this wasn't a named struct,set the out length to zero
            *len = 0;
            ptr::null()
        })
}

unsafe fn ffi_record_vec(fields: *const *mut Record, len: libc::size_t) -> Vec<Record> {
    let mut boxed_fields = Vec::with_capacity(len as usize);
    for i in 0..len {
        boxed_fields.push(*Box::from_raw(*fields.add(i)));
    }

    boxed_fields
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_insert_cmd(table: libc::size_t, rec: *mut Record) -> *mut UpdCmd {
    let rec = Box::from_raw(rec);
    Box::into_raw(Box::new(UpdCmd::Insert(RelIdentifier::RelId(table), *rec)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_insert_or_update_cmd(
    table: libc::size_t,
    rec: *mut Record,
) -> *mut UpdCmd {
    Box::into_raw(Box::new(UpdCmd::InsertOrUpdate(
        RelIdentifier::RelId(table),
        *Box::from_raw(rec),
    )))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delete_val_cmd(
    table: libc::size_t,
    rec: *mut Record,
) -> *mut UpdCmd {
    let rec = Box::from_raw(rec);
    Box::into_raw(Box::new(UpdCmd::Delete(RelIdentifier::RelId(table), *rec)))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_delete_key_cmd(
    table: libc::size_t,
    rec: *mut Record,
) -> *mut UpdCmd {
    let rec = Box::from_raw(rec);
    Box::into_raw(Box::new(UpdCmd::DeleteKey(
        RelIdentifier::RelId(table),
        *rec,
    )))
}

#[no_mangle]
pub unsafe extern "C" fn ddlog_modify_cmd(
    table: libc::size_t,
    key: *mut Record,
    values: *mut Record,
) -> *mut UpdCmd {
    let key = Box::from_raw(key);
    let values = Box::from_raw(values);
    Box::into_raw(Box::new(UpdCmd::Modify(
        RelIdentifier::RelId(table),
        *key,
        *values,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::bigint::ToBigInt;

    /// Test the `ddlog_dump_record` C API function.
    #[test]
    fn dump_record() {
        // Some very basic checks. Not all variants are covered at this
        // point.
        let checks = [
            (Record::Bool(true), "true"),
            (Record::Bool(false), "false"),
            (Record::Int(12345.to_bigint().unwrap()), "12345"),
            (Record::String("a-\0-byte".to_string()), "\"a-\\u{0}-byte\""),
        ];

        for check in &checks {
            let ptr = unsafe { ddlog_dump_record(&check.0) };
            assert!(!ptr.is_null());

            let actual = unsafe { CString::from_raw(ptr) };
            let expected = CString::new(check.1).unwrap();
            assert_eq!(actual, expected);
        }
    }

    /// Test `_with_length` C API functions.
    #[test]
    fn strings_with_length1() {
        unsafe {
            let string1 = ddlog_string_with_length("pod1".as_ptr() as *const i8, "pod1".len());
            let string2 = ddlog_string_with_length("ns1".as_ptr() as *const i8, "ns1".len());
            let strings = &[string1, string2];
            let structure = ddlog_struct_with_length(
                "k8spolicy.Pod".as_ptr() as *const i8,
                "k8spolicy.Pod".len(),
                strings.as_ptr(),
                strings.len(),
            );

            assert_eq!(
                CString::from(CStr::from_ptr(ddlog_dump_record(structure)))
                    .into_string()
                    .unwrap(),
                "k8spolicy.Pod{\"pod1\", \"ns1\"}".to_string()
            );

            ddlog_free(structure);
        }
    }

    #[test]
    fn strings_with_length2() {
        unsafe {
            let string1 = ddlog_string_with_length(std::ptr::null(), 0);
            let boolean = ddlog_bool(true);
            let fields = &[string1, boolean];
            let structure = ddlog_struct_static_cons_with_length(
                "Cons".as_ptr() as *const i8,
                "Cons".len(),
                fields.as_ptr(),
                fields.len(),
            );

            assert_eq!(
                CString::from(CStr::from_ptr(ddlog_dump_record(structure)))
                    .into_string()
                    .unwrap(),
                "Cons{\"\", true}".to_string()
            );

            ddlog_free(structure);
        }
    }
}
