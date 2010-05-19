//! SXO product types — `Dialect`, `SxoError`; no IR or Number.
//!
//! Kernel wire types come from `athena-types` (re-exported below).

#![deny(missing_docs)]

mod dialect;
mod error;
mod eval_outcome;

pub use athena_types::{Diagnostic, DiagnosticCode, ResultId, Severity, SourceSpan, TermId};
pub use dialect::Dialect;
pub use error::SxoError;
pub use eval_outcome::EvalOutcome;

/// SXO product version (semver string for N-API / packages).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
