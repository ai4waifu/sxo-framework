//! Thin shared types for SXO frontends and hosts.
//!
//! Product-layer only: dialect detection and localized errors.
//! Kernel wire types come from `euler-types` (re-exported below).

#![deny(missing_docs)]

mod dialect;
mod error;

pub use dialect::{Dialect, detect_dialect};
pub use error::SxoError;
pub use euler_types::{Diagnostic, DiagnosticCode, Severity, SourceSpan, TermId};

/// Crate version string aligned with Cargo package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
