//! Host session: dialect parse/render wired to Athena evaluation.

use athena::{
    AthenaEngine, CalculusRequest, CalculusResult, CalculusValue, Diagnostic, DomainRequest, DomainResult, Term,
    calculus_result_bridge_term, try_calculus_request,
};
use sxo_dialect_mathematica::{self as mathematica, WExpr};
use sxo_dialect_matlab as matlab;
use sxo_types::{Dialect, SxoError, detect_dialect};

/// SXO host session: dialect crates + Athena math.
#[derive(Debug, Default, Clone)]
pub struct Session {
    /// Preferred render dialect when callers do not override.
    #[allow(dead_code)]
    pub default_dialect: Dialect,
}

impl Session {
    /// Create a session with `Auto` as the default dialect preference.
    pub fn new() -> Self {
        Self { default_dialect: Dialect::Auto }
    }

    /// Resolve `Auto` against `input`, otherwise return `dialect`.
    pub fn resolve_dialect(&self, input: &str, dialect: Dialect) -> Dialect {
        match dialect {
            Dialect::Auto => detect_dialect(input),
            other => other,
        }
    }

    fn math(&self) -> AthenaEngine {
        AthenaEngine::new()
    }

    fn try_evaluate_calculus(&self, term: &Term) -> Option<Result<Term, Diagnostic>> {
        let request = try_calculus_request(term).map(DomainRequest::Calculus)?;
        Some(self.math().execute_domain(request).and_then(|r| {
            match r {
                DomainResult::Calculus(c) => Ok(calculus_result_bridge_term(&c)),
                other => Err(Diagnostic::new(athena::DiagnosticCode::TypeMismatch)
                    .detail("domain", "calculus")
                    .detail("operation", "try_evaluate_calculus")
                    .detail("got", format!("{other:?}"))),
            }
        }))
    }

    /// Evaluate a term, preferring calculus domain lowering.
    pub fn evaluate(&self, expr: &Term) -> Term {
        if let Some(Ok(term)) = self.try_evaluate_calculus(expr) {
            return term;
        }
        self.math().evaluate_term(expr)
    }

    /// Differentiate via Athena calculus domain dispatch.
    pub fn differentiate_term(&self, expr: &Term, var: &str) -> Term {
        match self.execute_domain(DomainRequest::Calculus(CalculusRequest::Derivative {
            expression: expr.clone(),
            variable: var.to_string(),
            order: athena::DerivativeOrder::First,
            assumptions: athena::AssumptionSet::empty(),
        })) {
            Ok(DomainResult::Calculus(r)) => calculus_result_bridge_term(&r),
            _ => self.math().differentiate_term(expr, var),
        }
    }

    /// Domain dispatch through Athena.
    pub fn execute_domain(&self, request: DomainRequest) -> Result<DomainResult, Diagnostic> {
        self.math().execute_domain(request)
    }

    /// `Simplify` builtin on a term.
    pub fn simplify_term(&self, expr: &Term) -> Term {
        self.math().simplify_term(expr)
    }

    /// Parse Wolfram text into MMA [`WExpr`] (no evaluate).
    pub fn parse_mathematica(&self, input: &str) -> Result<WExpr, SxoError> {
        mathematica::parse_mathematica(input)
    }

    /// MMA form → engine term.
    pub fn from_mathematica(&self, w: &WExpr) -> Term {
        mathematica::wexpr_to_term(w)
    }

    /// Engine term → MMA form.
    pub fn to_mathematica(&self, t: &Term) -> WExpr {
        mathematica::term_to_wexpr(t)
    }

    /// Parse Wolfram, lower, evaluate.
    #[allow(dead_code)]
    pub fn evaluate_mathematica(&self, input: &str) -> Result<Term, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.evaluate(&self.from_mathematica(&w)))
    }

    /// Differentiate Wolfram input.
    #[allow(dead_code)]
    pub fn d_mathematica(&self, input: &str, var: &str) -> Result<Term, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.differentiate_term(&self.from_mathematica(&w), var))
    }

    /// Render a term as Wolfram text.
    pub fn render_as_wolfram(&self, t: &Term) -> String {
        mathematica::render(&self.to_mathematica(t))
    }

    /// Parse MATLAB text into a term (no evaluate).
    pub fn parse_matlab(&self, input: &str) -> Result<Term, SxoError> {
        matlab::parse_matlab(input)
    }

    /// Parse MATLAB and evaluate.
    #[allow(dead_code)]
    pub fn evaluate_matlab(&self, input: &str) -> Result<Term, SxoError> {
        Ok(self.evaluate(&self.parse_matlab(input)?))
    }

    /// Differentiate MATLAB input.
    #[allow(dead_code)]
    pub fn d_matlab(&self, input: &str, var: &str) -> Result<Term, SxoError> {
        Ok(self.differentiate_term(&self.parse_matlab(input)?, var))
    }

    /// Render a term as MATLAB text.
    pub fn render_as_matlab(&self, t: &Term) -> String {
        matlab::render_matlab(t)
    }

    /// Try dialect `Plot` / `plot` → SVG via Athena sampling + Apollo.
    pub fn try_plot_svg(&self, term: &Term, dialect: Dialect) -> Option<Result<String, SxoError>> {
        match dialect {
            Dialect::Mathematica => mathematica::try_plot_svg(term),
            Dialect::Matlab => matlab::try_plot_svg(term),
            Dialect::SimpleMath | Dialect::Auto => None,
        }
    }

    /// Convenience: indefinite integral via calculus domain.
    #[allow(dead_code)]
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
}
