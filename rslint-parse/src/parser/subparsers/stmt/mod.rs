//! Sub parsers responsible for parsing JavaScript statements.  
//! Each sub parser handles a specific type of statement.  
//! The purpose of sub parsers is the ability to easily unit test each possible case and not clutter a single file.

pub mod block;
pub mod break_continue;
pub mod expr;
pub mod r#if;
pub mod r#return;
pub mod stmt;
pub mod switch;
pub mod throw;
pub mod r#try;
pub mod variable;
pub mod r#while;
