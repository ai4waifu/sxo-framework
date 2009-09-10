//! SXO product types — `Dialect`, `SxoError`; no IR or Number.
//!
//! Kernel wire types come from `athena-types` (re-exported below).

#![deny(missing_docs)]

mod dialect;
mod error;

pub use athena_types::{Diagnostic, DiagnosticCode, Severity, SourceSpan, TermId};
pub use dialect::{Dialect, detect_dialect};
pub use error::SxoError;

/// SXO product version (semver string for N-API / packages).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
