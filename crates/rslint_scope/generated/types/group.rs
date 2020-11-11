#![allow(
    path_statements,
    //unused_imports,
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
    clippy::match_single_binding
)]

// Required for #[derive(Serialize, Deserialize)].
use ::serde::Deserialize;
use ::serde::Serialize;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;

use crate::string_append_str;
use crate::string_append;
use crate::std_usize;
use crate::closure;

//
// use crate::ddlog_std;

pub fn all<K: crate::Val,V: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, bool>>) -> bool
{   for ref x in g.iter() {
        if (!f.call(x)) {
            return false
        } else {
            ()
        }
    };
    true
}
pub fn any<K: crate::Val,V: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, bool>>) -> bool
{   for ref x in g.iter() {
        if f.call(x) {
            return true
        } else {
            ()
        }
    };
    false
}
pub fn arg_max<K: crate::Val,V: crate::Val,A: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, A>>) -> V
{   let ref mut max_arg: V = crate::ddlog_std::first::<K, V>(g);
    let ref mut max_val: A = f.call((&crate::ddlog_std::first::<K, V>(g)));
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
pub fn arg_min<K: crate::Val,V: crate::Val,A: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, A>>) -> V
{   let ref mut min_arg: V = crate::ddlog_std::first::<K, V>(g);
    let ref mut min_val: A = f.call((&crate::ddlog_std::first::<K, V>(g)));
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
pub fn count<K: crate::Val,V: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, bool>>) -> u64
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
pub fn filter<K: crate::Val,V: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, bool>>) -> crate::ddlog_std::Vec<V>
{   let ref mut res: crate::ddlog_std::Vec<V> = crate::ddlog_std::vec_empty();
    for ref x in g.iter() {
        if f.call(x) {
            crate::ddlog_std::push::<V>(res, x)
        } else {
            ()
        }
    };
    (*res).clone()
}
pub fn filter_map<K: crate::Val,V1: crate::Val,V2: crate::Val>(g: & crate::ddlog_std::Group<K, V1>, f: & Box<dyn closure::Closure<*const V1, crate::ddlog_std::Option<V2>>>) -> crate::ddlog_std::Vec<V2>
{   let ref mut res: crate::ddlog_std::Vec<V2> = crate::ddlog_std::vec_empty();
    for ref x in g.iter() {
        match f.call(x) {
            crate::ddlog_std::Option::None{} => (),
            crate::ddlog_std::Option::Some{x: ref y} => crate::ddlog_std::push::<V2>(res, y)
        }
    };
    (*res).clone()
}
pub fn find<K: crate::Val,V: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<*const V, bool>>) -> crate::ddlog_std::Option<V>
{   for ref x in g.iter() {
        if f.call(x) {
            return (crate::ddlog_std::Option::Some{x: (*x).clone()})
        } else {
            ()
        }
    };
    (crate::ddlog_std::Option::None{})
}
pub fn flatmap<K: crate::Val,V1: crate::Val,V2: crate::Val>(g: & crate::ddlog_std::Group<K, V1>, f: & Box<dyn closure::Closure<*const V1, crate::ddlog_std::Vec<V2>>>) -> crate::ddlog_std::Vec<V2>
{   let ref mut res: crate::ddlog_std::Vec<V2> = crate::ddlog_std::vec_empty();
    for ref x in g.iter() {
        crate::ddlog_std::append::<V2>(res, (&f.call(x)))
    };
    (*res).clone()
}
pub fn fold<K: crate::Val,V: crate::Val,B: crate::Val>(g: & crate::ddlog_std::Group<K, V>, f: & Box<dyn closure::Closure<(*const B, *const V), B>>, initializer: & B) -> B
{   let ref mut res: B = (*initializer).clone();
    for ref x in g.iter() {
        (*res) = f.call((res, x))
    };
    (*res).clone()
}
pub fn map<K: crate::Val,V1: crate::Val,V2: crate::Val>(g: & crate::ddlog_std::Group<K, V1>, f: & Box<dyn closure::Closure<*const V1, V2>>) -> crate::ddlog_std::Vec<V2>
{   let ref mut res: crate::ddlog_std::Vec<V2> = crate::ddlog_std::vec_with_capacity((&crate::ddlog_std::count::<K, V1>(g)));
    for ref x in g.iter() {
        crate::ddlog_std::push::<V2>(res, (&f.call(x)))
    };
    (*res).clone()
}