//! Product facade: parse → bridge [`Term`] → evaluate/render.
//!
//! Math semantics live in **`euler`**; this type wires oak frontends and transitional `weval`.

use crate::{
    diff,
    expr::Expr,
    lowering::{KernelTerm, lower_to_kernel},
    mma_bridge::{term_to_wexpr, wexpr_to_term},
    parse_math::parse_mathematica,
    parse_matlab::parse_matlab,
    render::{render_mathematica, render_simple_math},
    render_matlab::render_matlab,
    render_wexpr::render_wexpr,
    simplify,
    term::Term,
    weval,
    wexpr::WExpr,
};
use sxo_types::{Dialect, SxoError, detect_dialect};

/// SXO engine: frontend parse, bridge IR, host-facing evaluate/render.
#[derive(Debug, Default, Clone)]
pub struct SxoEngine {
    /// Preferred render dialect when callers do not override.
    pub default_dialect: Dialect,
}

impl SxoEngine {
    /// Create an engine with `Auto` as the default dialect preference.
    pub fn new() -> Self {
        Self { default_dialect: Dialect::Auto }
    }

    /// Create an engine with an explicit default dialect.
    pub fn with_dialect(dialect: Dialect) -> Self {
        Self { default_dialect: dialect }
    }

    /// Resolve `Auto` against `input`, otherwise return `dialect`.
    pub fn resolve_dialect(&self, input: &str, dialect: Dialect) -> Dialect {
        match dialect {
            Dialect::Auto => detect_dialect(input),
            other => other,
        }
    }

    /// Lower bridge [`Term`] into Euler kernel IR.
    pub fn lower_to_kernel(&self, term: &Term) -> Result<KernelTerm, SxoError> {
        lower_to_kernel(term)
    }

    /// Evaluate bridge [`Term`] (transitional builtin evaluator).
    pub fn evaluate(&self, expr: &Term) -> Term {
        weval::evaluate(expr)
    }

    /// Differentiate bridge [`Term`].
    pub fn differentiate_term(&self, expr: &Term, var: &str) -> Term {
        self.evaluate(&weval::differentiate(expr, var))
    }

    /// `Simplify` builtin on bridge [`Term`].
    pub fn simplify_term(&self, expr: &Term) -> Term {
        self.evaluate(&Term::app("Simplify", vec![expr.clone()]))
    }

    // ---- Mathematica frontend (WExpr) ----

    /// Parse Wolfram text into MMA frontend [`WExpr`] (no evaluate).
    pub fn parse_mathematica(&self, input: &str) -> Result<WExpr, SxoError> {
        parse_mathematica(input)
    }

    /// MMA frontend form → bridge [`Term`].
    pub fn from_mathematica(&self, w: &WExpr) -> Term {
        wexpr_to_term(w)
    }

    /// Bridge [`Term`] → MMA frontend form.
    pub fn to_mathematica(&self, t: &Term) -> WExpr {
        term_to_wexpr(t)
    }

    /// Parse Wolfram, lower to [`Term`], evaluate.
    pub fn evaluate_mathematica(&self, input: &str) -> Result<Term, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.evaluate(&self.from_mathematica(&w)))
    }

    /// Differentiate Wolfram input (via bridge [`Term`]).
    pub fn d_mathematica(&self, input: &str, var: &str) -> Result<Term, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.differentiate_term(&self.from_mathematica(&w), var))
    }

    /// Render bridge [`Term`] as Wolfram text (via MMA frontend form).
    pub fn render_as_wolfram(&self, t: &Term) -> String {
        render_wexpr(&self.to_mathematica(t))
    }

    // ---- MATLAB frontend ----

    /// Parse MATLAB text into bridge [`Term`] (no evaluate).
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

    /// Render bridge [`Term`] as MATLAB text.
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
