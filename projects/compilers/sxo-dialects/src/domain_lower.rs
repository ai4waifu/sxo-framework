//! Lower dialect Form / Athena bridge [`Term`] into [`DomainRequest`].
//!
//! Math stays in Athena. This module only recognizes already-decoded Term shapes
//! produced by oak parse + Form bridges (no source-text parse).

use athena::{
    AssumptionSet, CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder, Diagnostic, DomainRequest, DomainResult,
    LimitApproach, LimitDirection, Term, calculus_result_bridge_term, try_calculus_request,
};

/// Attempt to lower a calculus head into [`DomainRequest::Calculus`].
pub fn lower_calculus_term(term: &Term) -> Option<DomainRequest> {
    try_calculus_request(term).map(DomainRequest::Calculus)
}

/// Evaluate via domain dispatch when the term is a recognized calculus form.
///
/// Returns `None` when the term is not a calculus domain request (caller falls
/// back to ordinary `evaluate_term`).
pub fn try_evaluate_calculus(
    term: &Term,
    execute: impl FnOnce(DomainRequest) -> Result<DomainResult, Diagnostic>,
) -> Option<Result<Term, Diagnostic>> {
    let request = lower_calculus_term(term)?;
    Some(execute(request).and_then(|r| match r {
        DomainResult::Calculus(c) => Ok(calculus_result_bridge_term(&c)),
        other => Err(Diagnostic::error(
            athena::DiagnosticCode::TypeMismatch,
            format!("expected Calculus domain result, got {other:?}"),
        )),
    }))
}

/// Helper builders used by tests and future dialect-specific shims.
pub fn derivative_request(expression: Term, variable: impl Into<String>) -> DomainRequest {
    DomainRequest::Calculus(CalculusRequest::Derivative {
        expression,
        variable: variable.into(),
        order: DerivativeOrder::First,
        assumptions: AssumptionSet::empty(),
    })
}

/// Build a two-sided finite/infinite limit request.
pub fn limit_request(expression: Term, variable: impl Into<String>, approach: LimitApproach) -> DomainRequest {
    DomainRequest::Calculus(CalculusRequest::Limit {
        expression,
        variable: variable.into(),
        approach,
        direction: LimitDirection::TwoSided,
        assumptions: AssumptionSet::empty(),
    })
}

/// Build a Taylor series request.
pub fn series_request(expression: Term, variable: impl Into<String>, center: Term, order: u32) -> DomainRequest {
    DomainRequest::Calculus(CalculusRequest::Series {
        expression,
        variable: variable.into(),
        center,
        order,
        assumptions: AssumptionSet::empty(),
    })
}
