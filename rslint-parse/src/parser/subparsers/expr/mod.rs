//! Sub parsers responsible for parsing JavaScript expressions.  
//! Each sub parser handles a specific type of expression which are recursively used with `parse_expr`.  
//! The purpose of sub parsers is the ability to easily unit test each possible case and not clutter a single file.

pub mod assign_expr;
pub mod binary_expr;
pub mod conditional_expr;
pub mod expr;
pub mod ident;
pub mod lhs_expr;
pub mod member_expr;
pub mod primary_expr;
pub mod primary_literals;
pub mod suffix;
pub mod unary_expr;
