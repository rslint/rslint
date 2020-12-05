#![allow(clippy::trivially_copy_pass_by_ref)]

//! Runtime support for DDlog closures.

use ::std::cmp::Ordering;
use ::std::fmt::Debug;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::hash::Hash;
use ::std::hash::Hasher;
use ::std::result::Result;

use ::serde::de::Error;
use ::serde::Deserialize;
use ::serde::Deserializer;
use ::serde::Serialize;
use ::serde::Serializer;

use crate::Val;

#[cfg(feature = "flatbuf")]
use flatbuffers as fbrt;

#[cfg(feature = "flatbuf")]
use crate::flatbuf;

#[cfg(feature = "flatbuf")]
use crate::flatbuf::ToFlatBuffer;

#[cfg(feature = "flatbuf")]
use crate::flatbuf::fb;

use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::differential_datalog::record::Record;

use ::abomonation::Abomonation;

/* DDlog's equivalent of Rust's `Fn` trait.  This is necessary, as Rust does not allow manual
 * implementations of `Fn` trait (until `unboxed_closures` and `fn_traits` features are
 * stabilized).  Otherwise, we would just derive `Fn` and add methods for comparison and hashing.
 */
pub trait Closure<Args, Output>: Send + Sync {
    fn call(&self, args: Args) -> Output;
    /* Returns pointers to function and captured arguments, for use in comparison methods. */
    fn internals(&self) -> (usize, usize);
    fn clone_dyn(&self) -> Box<dyn Closure<Args, Output>>;
    fn eq_dyn(&self, other: &dyn Closure<Args, Output>) -> bool;
    fn cmp_dyn(&self, other: &dyn Closure<Args, Output>) -> Ordering;
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn into_record_dyn(&self) -> Record;
    fn fmt_debug_dyn(&self, f: &mut Formatter) -> std::fmt::Result;
    fn fmt_display_dyn(&self, f: &mut Formatter) -> std::fmt::Result;
    fn serialize_dyn(&self) -> &dyn erased_serde::Serialize;
}

#[derive(Clone)]
pub struct ClosureImpl<Args, Output, Captured: Val> {
    pub description: &'static str,
    pub captured: Captured,
    pub f: fn(args: Args, captured: &Captured) -> Output,
}

impl<Args, Output, Captured: Debug + Val> Serialize for ClosureImpl<Args, Output, Captured> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }
}

/* Rust forces 'static trait bound on `Args` and `Output`, as the borrow checker is not smart
 * enough to realize that they are only used as arguments to `f`.
 */
impl<Args: Clone + 'static, Output: Clone + 'static, Captured: Debug + Val + Send + Sync>
    Closure<Args, Output> for ClosureImpl<Args, Output, Captured>
{
    fn call(&self, args: Args) -> Output {
        (self.f)(args, &self.captured)
    }

    fn clone_dyn(&self) -> Box<dyn Closure<Args, Output>> {
        Box::new((*self).clone()) as Box<dyn Closure<Args, Output>>
    }

    fn internals(&self) -> (usize, usize) {
        (
            self.f as *const (fn(Args, &Captured) -> Output) as usize,
            &self.captured as *const Captured as usize,
        )
    }

    fn eq_dyn(&self, other: &dyn Closure<Args, Output>) -> bool {
        /* Compare function pointers.  If equal, it is safe to compare captured variables. */
        let (other_f, other_captured) = other.internals();
        if (other_f == (self.f as *const (fn(Args, &Captured) -> Output) as usize)) {
            unsafe { *(other_captured as *const Captured) == self.captured }
        } else {
            false
        }
    }

    fn cmp_dyn(&self, other: &dyn Closure<Args, Output>) -> Ordering {
        let (other_f, other_captured) = other.internals();
        match ((self.f as *const (fn(Args, &Captured) -> Output) as usize).cmp(&other_f)) {
            Ordering::Equal => self
                .captured
                .cmp(unsafe { &*(other_captured as *const Captured) }),
            ord => ord,
        }
    }

    fn hash_dyn(&self, mut state: &mut dyn Hasher) {
        self.captured.hash(&mut state);
        (self.f as *const (fn(Args, &Captured) -> Output) as usize).hash(&mut state);
    }

    fn into_record_dyn(&self) -> Record {
        Record::String(format!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }

    fn fmt_debug_dyn(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }

    fn fmt_display_dyn(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }

    fn serialize_dyn(&self) -> &dyn erased_serde::Serialize {
        self as &dyn erased_serde::Serialize
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> Display for Box<dyn Closure<Args, Output>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_display_dyn(f)
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> Debug for Box<dyn Closure<Args, Output>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_debug_dyn(f)
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> PartialEq<&Self>
    for Box<dyn Closure<Args, Output>>
{
    fn eq(&self, other: &&Self) -> bool {
        self.eq_dyn(&***other)
    }
}

/* This extra impl is a workaround for compiler bug that fails to derive `PartialEq` for
 * structs that contain fields of type `Box<dyn Closure<>>`. See:
 * https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186 */
impl<Args: Clone + 'static, Output: Clone + 'static> PartialEq for Box<dyn Closure<Args, Output>> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(&**other)
    }
}
impl<Args: Clone + 'static, Output: Clone + 'static> Eq for Box<dyn Closure<Args, Output>> {}

impl<Args: Clone + 'static, Output: Clone + 'static> PartialOrd for Box<dyn Closure<Args, Output>> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_dyn(&**other))
    }
}
impl<Args: Clone + 'static, Output: Clone + 'static> Ord for Box<dyn Closure<Args, Output>> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(&**other)
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> Clone for Box<dyn Closure<Args, Output>> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone + Default> Default
    for Box<dyn Closure<Args, Output>>
{
    fn default() -> Self {
        Box::new(ClosureImpl {
            description: "default closure",
            captured: (),
            f: {
                fn __f<A, O: Default>(args: A, captured: &()) -> O {
                    O::default()
                };
                __f
            },
        })
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> Hash for Box<dyn Closure<Args, Output>> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.hash_dyn(state);
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> Serialize for Box<dyn Closure<Args, Output>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        erased_serde::serialize((self.serialize_dyn()), serializer)
    }
}

impl<'de, Args: 'static + Clone, Output: 'static + Clone> Deserialize<'de>
    for Box<dyn Closure<Args, Output>>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Err(D::Error::custom(
            "Deserialization of closures is not implemented.",
        ))
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> Mutator<Box<dyn Closure<Args, Output>>>
    for Record
{
    fn mutate(&self, x: &mut Box<dyn Closure<Args, Output>>) -> Result<(), String> {
        Err("'mutate' not implemented for closures.".to_string())
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> IntoRecord for Box<dyn Closure<Args, Output>> {
    fn into_record(self) -> Record {
        self.into_record_dyn()
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> FromRecord for Box<dyn Closure<Args, Output>> {
    fn from_record(val: &Record) -> Result<Self, String> {
        Err("'from_record' not implemented for closures.".to_string())
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> Abomonation
    for Box<dyn Closure<Args, Output>>
{
    unsafe fn entomb<W: std::io::Write>(&self, _write: &mut W) -> std::io::Result<()> {
        panic!("Closure::entomb: not implemented")
    }
    unsafe fn exhume<'a, 'b>(&'a mut self, _bytes: &'b mut [u8]) -> Option<&'b mut [u8]> {
        panic!("Closure::exhume: not implemented")
    }
    fn extent(&self) -> usize {
        panic!("Closure::extent: not implemented")
    }
}

#[cfg(feature = "flatbuf")]
impl<'a, Args: 'static + Clone, Output: 'static + Clone> flatbuf::FromFlatBuffer<&'a str>
    for Box<dyn Closure<Args, Output>>
{
    fn from_flatbuf(s: &'a str) -> Result<Self, String> {
        Err(format!("'from_flatbuf' not implemented for closures."))
    }
}

#[cfg(feature = "flatbuf")]
impl<'a, Args: 'static + Clone, Output: 'static + Clone> flatbuf::FromFlatBuffer<fb::__String<'a>>
    for Box<dyn Closure<Args, Output>>
{
    fn from_flatbuf(v: fb::__String<'a>) -> Result<Self, String> {
        Err(format!("'from_flatbuf' not implemented for closures."))
    }
}

#[cfg(feature = "flatbuf")]
impl<'b, Args: 'static + Clone, Output: 'static + Clone> ToFlatBuffer<'b>
    for Box<dyn Closure<Args, Output>>
{
    type Target = fbrt::WIPOffset<&'b str>;
    fn to_flatbuf(&self, fbb: &mut fbrt::FlatBufferBuilder<'b>) -> Self::Target {
        fbb.create_string(&format!("{}", self))
    }
}

#[cfg(feature = "flatbuf")]
impl<'b, Args: 'static + Clone, Output: 'static + Clone> flatbuf::ToFlatBufferTable<'b>
    for Box<dyn Closure<Args, Output>>
{
    type Target = fb::__String<'b>;
    fn to_flatbuf_table(
        &self,
        fbb: &mut fbrt::FlatBufferBuilder<'b>,
    ) -> fbrt::WIPOffset<Self::Target> {
        let v = self.to_flatbuf(fbb);
        fb::__String::create(fbb, &fb::__StringArgs { v: Some(v) })
    }
}

#[cfg(feature = "flatbuf")]
impl<'b, Args: 'static + Clone, Output: 'static + Clone> flatbuf::ToFlatBufferVectorElement<'b>
    for Box<dyn Closure<Args, Output>>
{
    type Target = <Self as flatbuf::ToFlatBuffer<'b>>::Target;

    fn to_flatbuf_vector_element(&self, fbb: &mut fbrt::FlatBufferBuilder<'b>) -> Self::Target {
        self.to_flatbuf(fbb)
    }
}

#[cfg(test)]
mod tests {
    use crate::closure::Closure;
    use crate::closure::ClosureImpl;
    use ::serde::Deserialize;
    use ::serde::Serialize;

    #[test]
    fn closure_test() {
        let closure1: ClosureImpl<(*const String, *const u32), Vec<String>, Vec<u64>> =
            ClosureImpl {
                description: "test closure 1",
                captured: vec![0, 1, 2, 3],
                f: {
                    fn __f(args: (*const String, *const u32), captured: &Vec<u64>) -> Vec<String> {
                        captured
                            .iter()
                            .map(|x| {
                                format!(
                                    "x: {}, arg0: {}, arg1: {}",
                                    x,
                                    unsafe { &*args.0 },
                                    unsafe { &*args.1 }
                                )
                            })
                            .collect()
                    };
                    __f
                },
            };

        let closure2: ClosureImpl<(*const String, *const u32), Vec<String>, String> = ClosureImpl {
            description: "test closure 1",
            captured: "Bar".to_string(),
            f: {
                fn __f(args: (*const String, *const u32), captured: &String) -> Vec<String> {
                    vec![format!(
                        "captured: {}, arg0: {}, arg1: {}",
                        captured,
                        unsafe { &*args.0 },
                        unsafe { &*args.1 }
                    )]
                };
                __f
            },
        };

        let ref arg1 = "bar".to_string();
        let ref arg2: u32 = 100;
        assert_eq!(
            closure1.call((arg1, arg2)),
            vec![
                "x: 0, arg0: bar, arg1: 100",
                "x: 1, arg0: bar, arg1: 100",
                "x: 2, arg0: bar, arg1: 100",
                "x: 3, arg0: bar, arg1: 100"
            ]
        );
        assert!(closure1.eq_dyn(&*closure1.clone_dyn()));
        assert!(closure2.eq_dyn(&*closure2.clone_dyn()));
        assert_eq!(closure1.eq_dyn(&closure2), false);
    }

    /* Make sure that auto-derives work for closures. */

    #[derive(Eq, PartialEq, Ord, Clone, Hash, PartialOrd, Default, Serialize, Deserialize)]
    pub struct IntClosure {
        pub f: Box<dyn Closure<*const i64, i64>>,
    }

    #[derive(Eq, PartialEq, Ord, Clone, Hash, PartialOrd, Serialize, Deserialize)]
    pub enum ClosureEnum {
        Enum1 {
            f: Box<dyn Closure<*const i64, i64>>,
        },
        Enum2 {
            f: Box<dyn Closure<(*mut Vec<String>, *const IntClosure), ()>>,
        },
    }
}
