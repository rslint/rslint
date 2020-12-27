#![allow(
    clippy::get_unwrap,
    clippy::missing_safety_doc,
    clippy::type_complexity,
    clippy::match_like_matches_macro
)]

mod callback;
mod ddlog;
mod profile;
mod profile_statistics;
mod replay;
mod valmap;
mod variable;

#[macro_use]
pub mod ddval;
pub mod program;

#[macro_use]
pub mod record;

#[cfg(test)]
mod test_record;

pub use callback::Callback;
pub use ddlog::DDlog;
pub use ddlog::DDlogConvert;
pub use replay::record_upd_cmds;
pub use replay::record_val_upds;
pub use replay::RecordReplay;
pub use valmap::DeltaMap;
