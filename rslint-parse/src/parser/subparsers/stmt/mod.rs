//! Sub parsers responsible for parsing JavaScript statements.  
//! Each sub parser handles a specific type of statement.  
//! The purpose of sub parsers is the ability to easily unit test each possible case and not clutter a single file.

pub mod variables;