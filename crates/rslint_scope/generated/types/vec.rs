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

use crate::closure::Closure;
use crate::ddlog_std;

pub fn vec_sort_by<A, B: Ord>(v: &mut ddlog_std::Vec<A>, f: &Box<dyn Closure<*const A, B>>) {
    v.x.sort_unstable_by_key(|x| f.call(x))
}

pub fn vec_arg_min<A: Clone, B: Ord>(
    v: &ddlog_std::Vec<A>,
    f: &Box<dyn Closure<*const A, B>>,
) -> ddlog_std::Option<A> {
    ddlog_std::Option::from(v.x.iter().min_by_key(|x| f.call(*x)).map(|x| x.clone()))
}

pub fn vec_arg_max<A: Clone, B: Ord>(
    v: &ddlog_std::Vec<A>,
    f: &Box<dyn Closure<*const A, B>>,
) -> ddlog_std::Option<A> {
    ddlog_std::Option::from(v.x.iter().max_by_key(|x| f.call(*x)).map(|x| x.clone()))
}

/* fn vec_arg_max<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> crate::ddlog_std::Option<A> */
/* fn vec_arg_min<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> crate::ddlog_std::Option<A> */
/* fn vec_sort_by<A: crate::Val,B: crate::Val>(v: &mut crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> () */
pub fn all<A: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, bool>>) -> bool
{   for x in v.iter() {
        if (!f.call(x)) {
            return false
        } else {
            ()
        }
    };
    true
}
pub fn any<A: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, bool>>) -> bool
{   for x in v.iter() {
        if f.call(x) {
            return true
        } else {
            ()
        }
    };
    false
}
pub fn arg_max<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> crate::ddlog_std::Option<A>
{   crate::vec::vec_arg_max(v, f)
}
pub fn arg_min<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> crate::ddlog_std::Option<A>
{   crate::vec::vec_arg_min(v, f)
}
pub fn count<A: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, bool>>) -> u64
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
pub fn filter<A: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, bool>>) -> crate::ddlog_std::Vec<A>
{   let ref mut res: crate::ddlog_std::Vec<A> = crate::ddlog_std::vec_empty();
    for x in v.iter() {
        if f.call(x) {
            crate::ddlog_std::push::<A>(res, x)
        } else {
            ()
        }
    };
    (*res).clone()
}
pub fn filter_map<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, crate::ddlog_std::Option<B>>>) -> crate::ddlog_std::Vec<B>
{   let ref mut res: crate::ddlog_std::Vec<B> = crate::ddlog_std::vec_empty();
    for x in v.iter() {
        match f.call(x) {
            crate::ddlog_std::Option::None{} => (),
            crate::ddlog_std::Option::Some{x: ref y} => crate::ddlog_std::push::<B>(res, y)
        }
    };
    (*res).clone()
}
pub fn find<A: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, bool>>) -> crate::ddlog_std::Option<A>
{   for x in v.iter() {
        if f.call(x) {
            return (crate::ddlog_std::Option::Some{x: (*x).clone()})
        } else {
            ()
        }
    };
    (crate::ddlog_std::Option::None{})
}
pub fn first<T: crate::Val>(vec: & crate::ddlog_std::Vec<T>) -> crate::ddlog_std::Option<T>
{   crate::ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X::<T>(vec, (&(0 as u64)))
}
pub fn flatmap<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, crate::ddlog_std::Vec<B>>>) -> crate::ddlog_std::Vec<B>
{   let ref mut res: crate::ddlog_std::Vec<B> = crate::ddlog_std::vec_empty();
    for x in v.iter() {
        crate::ddlog_std::append::<B>(res, (&f.call(x)))
    };
    (*res).clone()
}
pub fn fold<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<(*const B, *const A), B>>, initializer: & B) -> B
{   let ref mut res: B = (*initializer).clone();
    for x in v.iter() {
        (*res) = f.call((res, x))
    };
    (*res).clone()
}
pub fn last<T: crate::Val>(vec: & crate::ddlog_std::Vec<T>) -> crate::ddlog_std::Option<T>
{   crate::ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X::<T>(vec, (&(crate::ddlog_std::len_ddlog_std_Vec__X___Bitval64::<T>(vec).wrapping_sub((1 as u64)))))
}
pub fn map<A: crate::Val,B: crate::Val>(v: & crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> crate::ddlog_std::Vec<B>
{   let ref mut res: crate::ddlog_std::Vec<B> = crate::ddlog_std::vec_with_capacity((&crate::ddlog_std::len_ddlog_std_Vec__X___Bitval64::<A>(v)));
    for x in v.iter() {
        crate::ddlog_std::push::<B>(res, (&f.call(x)))
    };
    (*res).clone()
}
pub fn retain<A: crate::Val>(v: &mut crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, bool>>) -> ()
{   let ref mut del: crate::ddlog_std::s64 = (0 as i64);
    let ref mut len: u64 = crate::ddlog_std::len_ddlog_std_Vec__X___Bitval64::<A>(v);
    for i in crate::ddlog_std::range_vec((&(0 as i64)), (&((*len).clone() as i64)), (&(1 as i64))).iter() {
        {
            let ref mut x: A = crate::ddlog_std::unwrap_or_default_ddlog_std_Option__A_A::<A>((&crate::ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X::<A>(v, (&((*i).clone() as u64)))));
            if (!f.call(x)) {
                (*del) = ((*del).clone().wrapping_add((1 as i64)))
            } else {
                if ((&*del) > (&*(&(0 as i64)))) {
                    {
                        crate::ddlog_std::update_nth::<A>(v, (&(((*i).clone().wrapping_sub((*del).clone())) as u64)), x);
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
            crate::ddlog_std::truncate::<A>(v, (&((*len).clone().wrapping_sub(((*del).clone() as u64)))));
            ()
        }
    } else {
        ()
    }
}
pub fn sort_by<A: crate::Val,B: crate::Val>(v: &mut crate::ddlog_std::Vec<A>, f: & Box<dyn closure::Closure<*const A, B>>) -> ()
{   crate::vec::vec_sort_by(v, f)
}