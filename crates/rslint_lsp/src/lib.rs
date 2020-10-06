//! The RSLint Language Server.

#![deny(clippy::all)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

// Core definitions for the RSLint Language Server.
pub mod core;

// Definitions for implementation of the Language Server Protocol (LSP).
pub mod lsp;

// Providers for LSP features.
pub mod provider;

// Higher-level functionality not related to specific features, like document synchronization.
pub(crate) mod service;
