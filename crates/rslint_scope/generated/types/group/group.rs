#![allow(
    path_statements,
    unused_imports,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_parens,
    non_shorthand_field_patterns,
    dead_code,
    overflowing_literals,
    unreachable_patterns,
    unused_variables,
    clippy::missing_safety_doc,
    clippy::match_single_binding,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::needless_lifetimes,
    clippy::borrowed_box,
    clippy::map_clone,
    clippy::toplevel_ref_arg,
    clippy::double_parens,
    clippy::collapsible_if,
    clippy::clone_on_copy,
    clippy::unused_unit,
    clippy::deref_addrof,
    clippy::clone_on_copy,
    clippy::needless_return,
    clippy::op_ref,
    clippy::match_like_matches_macro,
    clippy::comparison_chain,
    clippy::len_zero,
    clippy::extra_unused_lifetimes
)]

use ::num::One;
use ::std::ops::Deref;

use ::differential_dataflow::collection;
use ::timely::communication;
use ::timely::dataflow::scopes;
use ::timely::worker;

use ::ddlog_derive::{FromRecord, IntoRecord, Mutator};
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::program::Weight;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;


// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;


pub fn all<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, bool>>) -> bool
{   for ref x in g.iter() {
        if (!f.call(x)) {
            return false
        } else {
            ()
        }
    };
    true
}
pub fn any<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, bool>>) -> bool
{   for ref x in g.iter() {
        if f.call(x) {
            return true
        } else {
            ()
        }
    };
    false
}
pub fn arg_max<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, A>>) -> V
{   let ref mut max_arg: V = ddlog_std::first::<K, V>(g);
    let ref mut max_val: A = f.call((&ddlog_std::first::<K, V>(g)));
    for ref x in g.iter() {
        {
            let ref mut v: A = f.call(x);
            if ((&*v) > (&*max_val)) {
                {
                    (*max_val) = (*v).clone();
                    (*max_arg) = (*x).clone();
                    ()
                }
            } else {
                ()
            }
        }
    };
    (*max_arg).clone()
}
pub fn arg_min<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, A>>) -> V
{   let ref mut min_arg: V = ddlog_std::first::<K, V>(g);
    let ref mut min_val: A = f.call((&ddlog_std::first::<K, V>(g)));
    for ref x in g.iter() {
        {
            let ref mut v: A = f.call(x);
            if ((&*v) < (&*min_val)) {
                {
                    (*min_val) = (*v).clone();
                    (*min_arg) = (*x).clone();
                    ()
                }
            } else {
                ()
            }
        }
    };
    (*min_arg).clone()
}
pub fn count<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, bool>>) -> u64
{   let ref mut cnt: u64 = (0 as u64);
    for ref x in g.iter() {
        if f.call(x) {
            (*cnt) = ((*cnt).clone().wrapping_add((1 as u64)))
        } else {
            ()
        }
    };
    (*cnt).clone()
}
pub fn filter<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, bool>>) -> ddlog_std::Vec<V>
{   let ref mut res: ddlog_std::Vec<V> = ddlog_std::vec_empty();
    for ref x in g.iter() {
        if f.call(x) {
            ddlog_std::push::<V>(res, x)
        } else {
            ()
        }
    };
    (*res).clone()
}
pub fn filter_map<K: ::ddlog_rt::Val,V1: ::ddlog_rt::Val,V2: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V1>, f: & Box<dyn ddlog_rt::Closure<*const V1, ddlog_std::Option<V2>>>) -> ddlog_std::Vec<V2>
{   let ref mut res: ddlog_std::Vec<V2> = ddlog_std::vec_empty();
    for ref x in g.iter() {
        match f.call(x) {
            ddlog_std::Option::None{} => (),
            ddlog_std::Option::Some{x: ref y} => ddlog_std::push::<V2>(res, y)
        }
    };
    (*res).clone()
}
pub fn find<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<*const V, bool>>) -> ddlog_std::Option<V>
{   for ref x in g.iter() {
        if f.call(x) {
            return (ddlog_std::Option::Some{x: (*x).clone()})
        } else {
            ()
        }
    };
    (ddlog_std::Option::None{})
}
pub fn flatmap<K: ::ddlog_rt::Val,V1: ::ddlog_rt::Val,V2: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V1>, f: & Box<dyn ddlog_rt::Closure<*const V1, ddlog_std::Vec<V2>>>) -> ddlog_std::Vec<V2>
{   let ref mut res: ddlog_std::Vec<V2> = ddlog_std::vec_empty();
    for ref x in g.iter() {
        ddlog_std::append::<V2>(res, (&f.call(x)))
    };
    (*res).clone()
}
pub fn fold<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V>, f: & Box<dyn ddlog_rt::Closure<(*const B, *const V), B>>, initializer: & B) -> B
{   let ref mut res: B = (*initializer).clone();
    for ref x in g.iter() {
        (*res) = f.call((res, x))
    };
    (*res).clone()
}
pub fn map<K: ::ddlog_rt::Val,V1: ::ddlog_rt::Val,V2: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, V1>, f: & Box<dyn ddlog_rt::Closure<*const V1, V2>>) -> ddlog_std::Vec<V2>
{   let ref mut res: ddlog_std::Vec<V2> = ddlog_std::vec_with_capacity((&ddlog_std::count::<K, V1>(g)));
    for ref x in g.iter() {
        ddlog_std::push::<V2>(res, (&f.call(x)))
    };
    (*res).clone()
}