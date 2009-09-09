//! Product-layer error; wraps Euler diagnostics when present.

use euler_types::Diagnostic;

/// Error from parse, lowering, or host operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SxoError {
    /// Localized or contextual message for the host.
    pub message: String,
    /// Stable Euler code (`EULER_*`) when the failure originated in the kernel.
    pub euler_code: Option<String>,
}

impl SxoError {
    /// Create a product error without a kernel code.
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), euler_code: None }
    }

    /// Map a kernel diagnostic into a product error (preserves stable code).
    pub fn from_diagnostic(d: Diagnostic) -> Self {
        Self { message: d.detail, euler_code: Some(d.code.as_str().to_string()) }
    }
}

impl std::fmt::Display for SxoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = &self.euler_code {
            write!(f, "{code}: {}", self.message)
        }
        else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for SxoError {}
