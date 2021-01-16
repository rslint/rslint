//! Implementing `Record`-related traits for Rust arrays

use crate::record::{CollectionKind, FromRecord, IntoRecord, Mutator, Record};
use std::result::Result;

/// Implements `FromRecord`, `IntoRecord` and `Mutator` for arrays
// FIXME: Replace this with `min_const_generics` after Rust v1.50
//        https://github.com/rust-lang/rust/issues/74878
macro_rules! ddlog_array_traits {
    ($($length:literal),* $(,)?) => {
        $(
            impl<T: FromRecord + Default> FromRecord for [T; $length] {
                fn from_record(val: &Record) -> Result<Self, String> {
                    let vec = Vec::from_record(val)?;
                    let mut arr = <[T; $length]>::default();

                    if vec.len() != $length {
                        return Err(format!(
                            "cannot convert {:?} to array of length {}",
                            *val, $length
                        ));
                    };
                    let mut idx = 0;
                    for v in vec.into_iter() {
                        arr[idx] = v;
                        idx += 1;
                    }
                    Ok(arr)

                    // Simpler implementation that requires Rust 1.48:
                    // Vec::from_record(val)?.try_into().map_err(|_| {
                    //     format!("cannot convert {:?} to array of length {}", *val, $length)
                    // })
                }
            }

            impl<T: IntoRecord + Clone> IntoRecord for [T; $length] {
                fn into_record(self) -> Record {
                    Record::Array(
                        CollectionKind::Vector,
                        self.iter().cloned().map(IntoRecord::into_record).collect(),
                    )
                }
            }

            impl<T: FromRecord + Default> Mutator<[T; $length]> for Record {
                fn mutate(&self, array: &mut [T; $length]) -> Result<(), String> {
                    *array = <[T; $length]>::from_record(self)?;
                    Ok(())
                }
            }
        )*
    };
}

ddlog_array_traits! {
    0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
    30, 31, 32,
}
