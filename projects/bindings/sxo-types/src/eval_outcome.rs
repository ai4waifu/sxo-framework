//! Host evaluation outcome — Athena result contract axes for product surfaces.

use athena_types::ResultId;

/// One host evaluation: Session-local [`ResultId`] plus status / coverage names.
///
/// Lets callers distinguish exact completion, approximate-but-full, conditional,
/// residual unevaluated, and hard failure — without treating the original form as success.
///
/// Symbolic [`athena_types::TermId`] and diagnostic summaries are projected on demand from
/// the owning Athena Session (`results[result_id]`), not stored here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalOutcome {
    /// Athena result handle owned by the evaluation Session.
    pub result_id: ResultId,
    /// [`athena_types::ComputationStatus`] machine name.
    pub status: String,
    /// Coverage machine name (`Full` / `Partial` / `Unknown` / `Unsupported`).
    pub coverage: String,
}

impl EvalOutcome {
    /// Build from Athena result axes (no eager term or diagnostic projection).
    pub fn new(result_id: ResultId, status: impl Into<String>, coverage: impl Into<String>) -> Self {
        Self { result_id, status: status.into(), coverage: coverage.into() }
    }
}
