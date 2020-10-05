//! Core definitions for the RSLint Language Server.

// Core functionality related to documents.
pub mod document;

// Core definitions related to runtime errors.
pub(crate) mod error;

// Core definitions related to language types for documents.
pub mod language;

// Core definitions related to the LSP server session.
pub mod session;
