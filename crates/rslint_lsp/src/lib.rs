//! The RSLint Language Server.

#![deny(clippy::all)]
#![deny(unsafe_code)]

#[allow(dead_code)] // temporary
pub(crate) mod util;

// Core definitions for the RSLint Language Server.
pub mod core;

// Definitions for implementation of the Language Server Protocol (LSP).
pub mod lsp;

// Providers for LSP features.
pub mod provider;

// Higher-level functionality not related to specific features, like document synchronization.
pub(crate) mod service;
