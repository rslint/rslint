//! All of the groups of built in rules in the linter.

pub mod ddlog;
pub mod errors;
pub mod style;

pub use ddlog::ddlog;
pub use errors::errors;
pub use style::style;

/// Macro for easily making a rule group hashmap.
/// This will call `::new()` on each rule.  
#[macro_export]
macro_rules! group {
    ($(#[$description:meta])* $groupname:ident, $($path:ident::$rule:ident),* $(,)?) => {
        use $crate::CstRule;
        $(
            mod $path;
            pub use $path::$rule;
        )*

        $(#[$description])*
        pub fn $groupname() -> Vec<Box<dyn CstRule>> {
            vec![$(Box::new($rule::new()) as Box<dyn CstRule>),*]
        }
    };
}
