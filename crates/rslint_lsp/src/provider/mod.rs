//! Providers for LSP features.

// Provider for LSP diagnostics.
pub(crate) mod diagnostics;

// Provider for LSP on-hover events.
pub(crate) mod hover;

// Provider for LSP completion events.
pub(crate) mod completion;

// Provider for LSP actions events.
pub(crate) mod actions;

// Provider for autocomplete for rslint config toml files
pub(crate) mod toml_completion;
