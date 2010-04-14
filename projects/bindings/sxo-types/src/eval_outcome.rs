//! Host evaluation outcome — Athena result contract axes for product surfaces.

use athena_types::TermId;

/// One host evaluation: symbolic term plus status / coverage / diagnostics.
///
/// Lets callers distinguish exact completion, approximate-but-full, conditional,
/// residual unevaluated, and hard failure — without treating the original form as success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalOutcome {
    /// Symbolic result term (may equal the input when residual / unsupported).
    pub term: TermId,
    /// [`athena_types::ComputationStatus`] machine name.
    pub status: String,
    /// Coverage machine name (`Full` / `Partial` / `Unknown` / `Unsupported`).
    pub coverage: String,
    /// Structured diagnostic summaries from the computation result.
    pub diagnostics: Vec<String>,
}

impl EvalOutcome {
    /// Build from Athena result axes.
    pub fn new(term: TermId, status: impl Into<String>, coverage: impl Into<String>, diagnostics: Vec<String>) -> Self {
        Self { term, status: status.into(), coverage: coverage.into(), diagnostics }
    }
}
