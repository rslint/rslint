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
    clippy::unknown_clippy_lints,
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

//use ::serde::de::DeserializeOwned;
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


use ddlog_rt::Closure;

pub fn vec_sort_by<A, B: Ord>(v: &mut ddlog_std::Vec<A>, f: &Box<dyn Closure<*const A, B>>) {
    v.sort_unstable_by_key(|x| f.call(x))
}

pub fn vec_arg_min<A: Clone, B: Ord>(
    v: &ddlog_std::Vec<A>,
    f: &Box<dyn Closure<*const A, B>>,
) -> ddlog_std::Option<A> {
    ddlog_std::Option::from(v.iter().min_by_key(|x| f.call(*x)).map(|x| x.clone()))
}

pub fn vec_arg_max<A: Clone, B: Ord>(
    v: &ddlog_std::Vec<A>,
    f: &Box<dyn Closure<*const A, B>>,
) -> ddlog_std::Option<A> {
    ddlog_std::Option::from(v.iter().max_by_key(|x| f.call(*x)).map(|x| x.clone()))
}

/* fn vec_arg_max<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> ddlog_std::Option<A> */
/* fn vec_arg_min<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> ddlog_std::Option<A> */
/* fn vec_sort_by<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: &mut ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> () */
pub fn all<A: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, bool>>) -> bool
{   for x in v.iter() {
        if (!f.call(x)) {
            return false
        } else {
            ()
        }
    };
    true
}
pub fn any<A: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, bool>>) -> bool
{   for x in v.iter() {
        if f.call(x) {
            return true
        } else {
            ()
        }
    };
    false
}
pub fn arg_max<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> ddlog_std::Option<A>
{   vec_arg_max(v, f)
}
pub fn arg_min<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> ddlog_std::Option<A>
{   vec_arg_min(v, f)
}
pub fn count<A: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, bool>>) -> u64
{   let ref mut cnt: u64 = (0 as u64);
    for x in v.iter() {
        if f.call(x) {
            (*cnt) = ((*cnt).clone().wrapping_add((1 as u64)))
        } else {
            ()
        }
    };
    (*cnt).clone()
}
pub fn filter<A: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, bool>>) -> ddlog_std::Vec<A>
{   let ref mut res: ddlog_std::Vec<A> = ddlog_std::vec_empty();
    for x in v.iter() {
        if f.call(x) {
            ddlog_std::push::<A>(res, x)
        } else {
            ()
        }
    };
    (*res).clone()
}
pub fn filter_map<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, ddlog_std::Option<B>>>) -> ddlog_std::Vec<B>
{   let ref mut res: ddlog_std::Vec<B> = ddlog_std::vec_empty();
    for x in v.iter() {
        match f.call(x) {
            ddlog_std::Option::None{} => (),
            ddlog_std::Option::Some{x: ref y} => ddlog_std::push::<B>(res, y)
        }
    };
    (*res).clone()
}
pub fn find<A: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, bool>>) -> ddlog_std::Option<A>
{   for x in v.iter() {
        if f.call(x) {
            return (ddlog_std::Option::Some{x: (*x).clone()})
        } else {
            ()
        }
    };
    (ddlog_std::Option::None{})
}
pub fn first<T: ::ddlog_rt::Val>(vec: & ddlog_std::Vec<T>) -> ddlog_std::Option<T>
{   ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X::<T>(vec, (&(0 as u64)))
}
pub fn flatmap<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, ddlog_std::Vec<B>>>) -> ddlog_std::Vec<B>
{   let ref mut res: ddlog_std::Vec<B> = ddlog_std::vec_empty();
    for x in v.iter() {
        ddlog_std::append::<B>(res, (&f.call(x)))
    };
    (*res).clone()
}
pub fn fold<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<(*const B, *const A), B>>, initializer: & B) -> B
{   let ref mut res: B = (*initializer).clone();
    for x in v.iter() {
        (*res) = f.call((res, x))
    };
    (*res).clone()
}
pub fn last<T: ::ddlog_rt::Val>(vec: & ddlog_std::Vec<T>) -> ddlog_std::Option<T>
{   ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X::<T>(vec, (&(ddlog_std::len_ddlog_std_Vec__X___Bitval64::<T>(vec).wrapping_sub((1 as u64)))))
}
pub fn map<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: & ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> ddlog_std::Vec<B>
{   let ref mut res: ddlog_std::Vec<B> = ddlog_std::vec_with_capacity((&ddlog_std::len_ddlog_std_Vec__X___Bitval64::<A>(v)));
    for x in v.iter() {
        ddlog_std::push::<B>(res, (&f.call(x)))
    };
    (*res).clone()
}
pub fn retain<A: ::ddlog_rt::Val>(v: &mut ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, bool>>) -> ()
{   let ref mut del: ddlog_std::s64 = (0 as i64);
    let ref mut len: u64 = ddlog_std::len_ddlog_std_Vec__X___Bitval64::<A>(v);
    for i in ddlog_std::range_vec((&(0 as i64)), (&((*len).clone() as i64)), (&(1 as i64))).iter() {
        {
            let ref mut x: A = ddlog_std::unwrap_or_default_ddlog_std_Option__A_A::<A>((&ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X::<A>(v, (&((*i).clone() as u64)))));
            if (!f.call(x)) {
                (*del) = ((*del).clone().wrapping_add((1 as i64)))
            } else {
                if ((&*del) > (&*(&(0 as i64)))) {
                    {
                        ddlog_std::update_nth::<A>(v, (&(((*i).clone().wrapping_sub((*del).clone())) as u64)), x);
                        ()
                    }
                } else {
                    ()
                }
            }
        }
    };
    if ((&*del) > (&*(&(0 as i64)))) {
        {
            ddlog_std::truncate::<A>(v, (&((*len).clone().wrapping_sub(((*del).clone() as u64)))));
            ()
        }
    } else {
        ()
    }
}
pub fn sort_by<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(v: &mut ddlog_std::Vec<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> ()
{   vec_sort_by(v, f)
}