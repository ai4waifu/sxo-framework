//! Product facade: parse → Athena [`Term`] → evaluate/render.
//!
//! Math semantics live in **`athena::AthenaEngine`**; this type wires oak frontends.

use athena::{AthenaEngine, CalculusRequest, CalculusResult, CalculusValue, Diagnostic, DomainRequest, DomainResult, Term};
use sxo_types::{Dialect, SxoError, detect_dialect};

use crate::{
    diff,
    domain_lower::try_evaluate_calculus,
    expr::Expr,
    lowering::{KernelTerm, lower_to_kernel},
    mma_bridge::{term_to_wexpr, wexpr_to_term},
    parse_math::parse_mathematica,
    parse_matlab::parse_matlab,
    render::{render_mathematica, render_simple_math},
    render_matlab::render_matlab,
    render_wexpr::render_wexpr,
    simplify,
    wexpr::WExpr,
};

/// SXO dialect frontend: oak parse, Form bridge, render; math via Athena.
#[derive(Debug, Default, Clone)]
pub struct SxoFrontend {
    /// Preferred render dialect when callers do not override.
    pub default_dialect: Dialect,
}

impl SxoFrontend {
    /// Create a frontend with `Auto` as the default dialect preference.
    pub fn new() -> Self {
        Self { default_dialect: Dialect::Auto }
    }

    /// Create a frontend with an explicit default dialect.
    pub fn with_dialect(dialect: Dialect) -> Self {
        Self { default_dialect: dialect }
    }

    fn math(&self) -> AthenaEngine {
        AthenaEngine::new()
    }

    /// Resolve `Auto` against `input`, otherwise return `dialect`.
    pub fn resolve_dialect(&self, input: &str, dialect: Dialect) -> Dialect {
        match dialect {
            Dialect::Auto => detect_dialect(input),
            other => other,
        }
    }

    /// Lower Athena [`Term`] into kernel arena IR.
    pub fn lower_to_kernel(&self, term: &Term) -> Result<KernelTerm, SxoError> {
        lower_to_kernel(term)
    }

    /// Evaluate Athena [`Term`], preferring calculus [`DomainRequest`] lowering.
    pub fn evaluate(&self, expr: &Term) -> Term {
        if let Some(Ok(term)) = try_evaluate_calculus(expr, |req| self.math().execute_domain(req)) {
            return term;
        }
        self.math().evaluate_term(expr)
    }

    /// Differentiate Athena [`Term`] through calculus domain dispatch.
    pub fn differentiate_term(&self, expr: &Term, var: &str) -> Term {
        match self.execute_domain(DomainRequest::Calculus(CalculusRequest::Derivative {
            expression: expr.clone(),
            variable: var.to_string(),
            order: athena::DerivativeOrder::First,
            assumptions: athena::AssumptionSet::empty(),
        })) {
            Ok(DomainResult::Calculus(r)) => athena::calculus_result_bridge_term(&r),
            _ => self.math().differentiate_term(expr, var),
        }
    }

    /// Domain dispatch through Athena.
    pub fn execute_domain(&self, request: DomainRequest) -> Result<DomainResult, Diagnostic> {
        self.math().execute_domain(request)
    }

    /// Convenience: indefinite integral via [`DomainRequest::Calculus`].
    pub fn integrate_term(&self, expr: &Term, var: &str) -> CalculusResult<CalculusValue> {
        match self
            .execute_domain(DomainRequest::Calculus(CalculusRequest::Integral {
                expression: expr.clone(),
                variable: var.to_string(),
                assumptions: athena::AssumptionSet::empty(),
            }))
            .expect("calculus Integral dispatch is infallible")
        {
            DomainResult::Calculus(c) => c,
            other => panic!("expected Calculus domain result, got {other:?}"),
        }
    }

    /// `Simplify` builtin on Athena [`Term`].
    pub fn simplify_term(&self, expr: &Term) -> Term {
        self.math().simplify_term(expr)
    }

    // ---- Mathematica frontend (WExpr) ----

    /// Parse Wolfram text into MMA frontend [`WExpr`] (no evaluate).
    pub fn parse_mathematica(&self, input: &str) -> Result<WExpr, SxoError> {
        parse_mathematica(input)
    }

    /// MMA frontend form → Athena [`Term`].
    pub fn from_mathematica(&self, w: &WExpr) -> Term {
        wexpr_to_term(w)
    }

    /// Athena [`Term`] → MMA frontend form.
    pub fn to_mathematica(&self, t: &Term) -> WExpr {
        term_to_wexpr(t)
    }

    /// Parse Wolfram, lower to [`Term`], evaluate.
    pub fn evaluate_mathematica(&self, input: &str) -> Result<Term, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.evaluate(&self.from_mathematica(&w)))
    }

    /// Differentiate Wolfram input (via Athena [`Term`]).
    pub fn d_mathematica(&self, input: &str, var: &str) -> Result<Term, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.differentiate_term(&self.from_mathematica(&w), var))
    }

    /// Render Athena [`Term`] as Wolfram text (via MMA frontend form).
    pub fn render_as_wolfram(&self, t: &Term) -> String {
        render_wexpr(&self.to_mathematica(t))
    }

    // ---- MATLAB frontend ----

    /// Parse MATLAB text into Athena [`Term`] (no evaluate).
    pub fn parse_matlab(&self, input: &str) -> Result<Term, SxoError> {
        parse_matlab(input)
    }

    /// Parse MATLAB and evaluate.
    pub fn evaluate_matlab(&self, input: &str) -> Result<Term, SxoError> {
        Ok(self.evaluate(&self.parse_matlab(input)?))
    }

    /// Differentiate MATLAB input.
    pub fn d_matlab(&self, input: &str, var: &str) -> Result<Term, SxoError> {
        Ok(self.differentiate_term(&self.parse_matlab(input)?, var))
    }

    /// Render Athena [`Term`] as MATLAB text.
    pub fn render_as_matlab(&self, t: &Term) -> String {
        render_matlab(t)
    }

    // ---- Legacy flat Expr (off-route) ----

    /// Differentiate legacy flat [`Expr`].
    pub fn differentiate(&self, expr: &Expr, var: &str) -> Expr {
        diff::differentiate(expr, var)
    }

    /// Simplify legacy flat [`Expr`].
    pub fn simplify(&self, expr: &Expr) -> Expr {
        simplify::simplify(expr)
    }

    /// Render legacy flat [`Expr`].
    pub fn render(&self, expr: &Expr, dialect: Dialect) -> String {
        match dialect {
            Dialect::Mathematica | Dialect::Matlab => render_mathematica(expr),
            Dialect::SimpleMath | Dialect::Auto => render_simple_math(expr),
        }
    }
}
