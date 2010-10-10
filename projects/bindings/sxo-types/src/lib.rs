//! SXO product types — `Dialect`, `SxoError`; no IR or Number.
//!
//! Kernel wire types come from `athena-types` (re-exported below).

#![deny(missing_docs)]

mod condition_summary;
mod dialect;
mod error;
mod eval_outcome;
mod evidence_summary;

pub use athena_types::{Diagnostic, DiagnosticCode, ResultId, Severity, SourceSpan, TermId};
pub use condition_summary::condition_summary;
pub use dialect::Dialect;
pub use error::SxoError;
pub use eval_outcome::EvalOutcome;
pub use evidence_summary::trusted_kernel_evidence_summary;

/// SXO product version (semver string for N-API / packages).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
