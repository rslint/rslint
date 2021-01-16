//! Implementing `Record`-related traits for Rust tuples

use crate::record::{FromRecord, IntoRecord, Mutator, Record};

/// Implements `FromRecord`, `IntoRecord` and `Mutator` for tuples
macro_rules! ddlog_tuple_traits {
    ($(($($generic:tt),* $(,)?)),* $(,)?) => {
        $(
            impl<$($generic: FromRecord),*> FromRecord for ($($generic,)*) {
                fn from_record(record: &Record) -> Result<Self, String> {
                    const TUPLE_LENGTH: usize = ddlog_tuple_traits!(@count $($generic),*);

                    match record {
                        Record::Tuple(args) if args.len() == TUPLE_LENGTH => {
                            Ok(ddlog_tuple_traits!(@from_record args [] [0] $($generic),*))
                        },
                        error => Err(format!("not a {}-tuple {:?}", TUPLE_LENGTH, *error)),
                    }
                }
            }

            impl<$($generic: IntoRecord),*> IntoRecord for ($($generic,)*) {
                fn into_record(self) -> Record {
                    #[allow(non_snake_case)]
                    let ($($generic,)*) = self;
                    Record::Tuple(vec![$($generic.into_record()),*])
                }
            }

            impl<$($generic: FromRecord),*> Mutator<($($generic,)*)> for Record {
                fn mutate(&self, tuple: &mut ($($generic,)*)) -> Result<(), String> {
                    *tuple = <($($generic,)*)>::from_record(self)?;
                    Ok(())
                }
            }
        )*
    };

    // Indexes the `args` vec returned from `Record::Tuple`, converts the inner types
    // using `FromRecord::from_record` and returns a tuple of the deserialized results
    (@from_record $args:ident [$($acc:tt)*] [$($counter:tt)*]) => {
        ($($acc)*)
    };
    (@from_record $args:ident [$($acc:tt)*] [$($counter:tt)*] $generic:tt) => {
        ddlog_tuple_traits!(
            @from_record
            $args
            [$($acc)* <$generic>::from_record(&$args[$($counter)*])?,]
            [$($counter)* + 1]
        )
    };
    (@from_record $args:ident [$($acc:tt)*] [$($counter:tt)*] $generic:tt, $($others:tt),*) => {
        ddlog_tuple_traits!(
            @from_record
            $args
            [$($acc)* <$generic>::from_record(&$args[$($counter)*])?,]
            [$($counter)* + 1]
            $($others),*
        )
    };

    // Allows counting within macros
    (@count) => { 0 };
    (@count $a:tt) => { 1 };
    (@count $a:tt, $($b:tt)+) => { 1 + ddlog_tuple_traits!(@count $($b)+) };
}

// Implement `FromRecord`, `IntoRecord` and `Mutator` for all tuple types of lengths
// zero through 30
ddlog_tuple_traits! {
    (),
    (T0,),
    (T0, T1),
    (T0, T1, T2),
    (T0, T1, T2, T3),
    (T0, T1, T2, T3, T4),
    (T0, T1, T2, T3, T4, T5),
    (T0, T1, T2, T3, T4, T5, T6),
    (T0, T1, T2, T3, T4, T5, T6, T7),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29),
}
