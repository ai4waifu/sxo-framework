//! Host session: dialect parse/render wired to Athena evaluation.

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session as AthenaSession,
    domains::{
        DomainExecutionContext, DomainRequest, DomainResult,
        calculus::{CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder, materialize_calculus_result_term},
    },
    types::{AssumptionSet, Diagnostic, TermId},
};
use sxo_dialect_mathematica::{self as mathematica, WExpr};
use sxo_dialect_matlab as matlab;
use sxo_types::{Dialect, SxoError, detect_dialect};

/// SXO host session: dialect crates + Athena math with persistent Own `Set` defs.
#[derive(Debug, Default)]
pub struct Session {
    /// Preferred render dialect when callers do not override.
    pub default_dialect: Dialect,
    /// Athena eval session (definitions persist across `evaluate` calls).
    math_session: RefCell<AthenaSession>,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self { default_dialect: self.default_dialect, math_session: RefCell::new(AthenaSession::new()) }
    }
}

impl Session {
    /// Create a session with `Auto` as the default dialect preference.
    pub fn new() -> Self {
        Self { default_dialect: Dialect::Auto, math_session: RefCell::new(AthenaSession::new()) }
    }

    /// Resolve `Auto` against `input`, otherwise return `dialect`.
    pub fn resolve_dialect(&self, input: &str, dialect: Dialect) -> Dialect {
        match dialect {
            Dialect::Auto => detect_dialect(input),
            other => other,
        }
    }

    fn math_engine(&self) -> AthenaEngine {
        AthenaEngine::new()
    }

    /// Evaluate a term through Athena (no Athena-term reverse-parse into calculus Goal).
    pub fn evaluate(&self, expr: TermId) -> TermId {
        self.math_session.borrow_mut().evaluate(expr)
    }

    /// Clear Athena Own symbol definitions for this host session.
    #[allow(dead_code)]
    pub fn clear_definitions(&self) {
        self.math_session.borrow_mut().clear_definitions();
    }

    /// Differentiate via Athena calculus domain dispatch.
    pub fn differentiate_term(&self, expr: TermId, var: &str) -> TermId {
        let mut ms = self.math_session.borrow_mut();
        let variable = ms.arena.symbols_mut().intern(var);
        match self.math_engine().execute_domain(
            &mut ms,
            DomainRequest::Calculus(CalculusRequest::Derivative {
                expression: expr,
                variable,
                order: DerivativeOrder::First,
                assumptions: AssumptionSet::empty(),
            }),
        ) {
            Ok(DomainResult::Calculus(r)) => {
                let mut dc = DomainExecutionContext::new(&mut ms);
                materialize_calculus_result_term(&mut dc, &r)
            }
            _ => self.math_engine().differentiate(&mut ms, expr, var),
        }
    }

    /// Domain dispatch through Athena.
    #[allow(dead_code)]
    pub fn execute_domain(&self, request: DomainRequest) -> Result<DomainResult, Diagnostic> {
        self.math_engine().execute_domain(&mut self.math_session.borrow_mut(), request)
    }

    /// `Simplify` builtin on a term.
    pub fn simplify_term(&self, expr: TermId) -> TermId {
        self.math_engine().simplify(&mut self.math_session.borrow_mut(), expr)
    }

    /// Parse Wolfram text into MMA [`WExpr`] (no evaluate).
    pub fn parse_mathematica(&self, input: &str) -> Result<WExpr, SxoError> {
        mathematica::parse_mathematica(input)
    }

    /// MMA form → session arena [`TermId`].
    pub fn lower_mathematica(&self, w: &WExpr) -> TermId {
        mathematica::lower_wexpr(&mut self.math_session.borrow_mut(), w)
    }

    /// Session arena [`TermId`] → MMA form.
    pub fn to_mathematica(&self, id: TermId) -> WExpr {
        mathematica::wexpr_from_session(&self.math_session.borrow(), id)
    }

    /// Parse Wolfram, lower via [`mathematica::lower_request`], execute.
    #[allow(dead_code)]
    pub fn evaluate_mathematica(&self, input: &str) -> Result<TermId, SxoError> {
        let w = self.parse_mathematica(input)?;
        let mut ms = self.math_session.borrow_mut();
        let request = mathematica::lower_request(&mut ms, &w);
        match self.math_engine().execute_request(&mut ms, request) {
            Ok(result_id) => Ok(ms
                .results
                .get(result_id)
                .and_then(|r| r.symbolic_term)
                .unwrap_or_else(|| mathematica::lower_wexpr(&mut ms, &w))),
            Err(d) => Err(SxoError::from_diagnostic(d)),
        }
    }

    /// Differentiate Wolfram input.
    #[allow(dead_code)]
    pub fn d_mathematica(&self, input: &str, var: &str) -> Result<TermId, SxoError> {
        let w = self.parse_mathematica(input)?;
        Ok(self.differentiate_term(self.lower_mathematica(&w), var))
    }

    /// Render a term as Wolfram text.
    pub fn render_as_wolfram(&self, id: TermId) -> String {
        mathematica::render(&self.to_mathematica(id))
    }

    /// Parse MATLAB text into a [`TermId`] (no evaluate).
    pub fn parse_matlab(&self, input: &str) -> Result<TermId, SxoError> {
        matlab::parse_matlab(&mut self.math_session.borrow_mut(), input)
    }

    /// Parse MATLAB, lift via [`matlab::lower_request`], execute.
    #[allow(dead_code)]
    pub fn evaluate_matlab(&self, input: &str) -> Result<TermId, SxoError> {
        let term = self.parse_matlab(input)?;
        let mut ms = self.math_session.borrow_mut();
        let request = matlab::lower_request(&mut ms, term);
        match self.math_engine().execute_request(&mut ms, request) {
            Ok(result_id) => Ok(ms.results.get(result_id).and_then(|r| r.symbolic_term).unwrap_or(term)),
            Err(d) => Err(SxoError::from_diagnostic(d)),
        }
    }

    /// Differentiate MATLAB input.
    #[allow(dead_code)]
    pub fn d_matlab(&self, input: &str, var: &str) -> Result<TermId, SxoError> {
        Ok(self.differentiate_term(self.parse_matlab(input)?, var))
    }

    /// Render a term as MATLAB text.
    pub fn render_as_matlab(&self, id: TermId) -> String {
        matlab::render_matlab(&self.math_session.borrow(), id)
    }

    /// Try dialect `Plot` / `plot` → SVG via Athena sampling + Apollo.
    pub fn try_plot_svg(&self, id: TermId, dialect: Dialect) -> Option<Result<String, SxoError>> {
        let mut ms = self.math_session.borrow_mut();
        match dialect {
            Dialect::Mathematica => mathematica::try_plot_svg(&mut ms, id),
            Dialect::Matlab => matlab::try_plot_svg(&mut ms, id),
            Dialect::SimpleMath | Dialect::Auto => None,
        }
    }

    /// Convenience: indefinite integral via calculus domain.
    #[allow(dead_code)]
    pub fn integrate_term(&self, expr: TermId, var: &str) -> CalculusResult<CalculusValue> {
        let mut ms = self.math_session.borrow_mut();
        let variable = ms.arena.symbols_mut().intern(var);
        match self
            .math_engine()
            .execute_domain(
                &mut ms,
                DomainRequest::Calculus(CalculusRequest::Integral {
                    expression: expr,
                    variable,
                    assumptions: AssumptionSet::empty(),
                }),
            )
            .expect("calculus Integral dispatch is infallible")
        {
            DomainResult::Calculus(c) => c,
            other => panic!("expected Calculus domain result, got {other:?}"),
        }
    }
}
