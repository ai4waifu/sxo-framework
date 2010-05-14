//! Host session: dialect parse/render wired to Athena evaluation.

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session as AthenaSession,
    api::AthenaRequest,
    domains::{
        DomainExecutionContext, DomainRequest, DomainResult,
        calculus::{CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder, materialize_calculus_result_term},
    },
    runtime::CoverageStatus,
    types::{AssumptionSet, ComputationStatus, Diagnostic, ResultId, TermId},
};
use sxo_dialect_mathematica::{self as mathematica, WolframForm};
use sxo_dialect_matlab as matlab;
use sxo_types::{Dialect, EvalOutcome, SxoError};

/// SXO host session: dialect crates + Athena math with persistent Own `Set` defs.
#[derive(Debug, Default)]
pub struct Session {
    /// Athena eval session (definitions persist across `evaluate` calls).
    math_session: RefCell<AthenaSession>,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        // Fresh Athena session: definitions are not shared across cloned host sessions.
        Self { math_session: RefCell::new(AthenaSession::new()) }
    }
}

impl Session {
    /// Create a host session with a fresh Athena math session.
    pub fn new() -> Self {
        Self { math_session: RefCell::new(AthenaSession::new()) }
    }

    fn math_engine(&self) -> AthenaEngine {
        AthenaEngine::new()
    }

    /// Borrow the underlying Athena session mutably.
    pub fn with_math_mut<R>(&self, f: impl FnOnce(&mut AthenaSession) -> R) -> R {
        f(&mut self.math_session.borrow_mut())
    }

    /// Borrow the underlying Athena session shared.
    pub fn with_math<R>(&self, f: impl FnOnce(&AthenaSession) -> R) -> R {
        f(&self.math_session.borrow())
    }

    /// Evaluate a term through Athena (no Athena-term reverse-parse into calculus Goal).
    ///
    /// Own `Set` bindings persist on this host session until cleared.
    /// Prefer [`Self::evaluate_matlab_form`] / [`Self::evaluate_wolfram_form`] for dialect surface.
    pub fn evaluate(&self, expr: TermId) -> Result<TermId, SxoError> {
        self.math_session.borrow_mut().evaluate(expr).map_err(SxoError::from_diagnostic)
    }

    /// Evaluate a held [`matlab::MatlabForm`] via [`matlab::lower_request`] on this session.
    pub fn evaluate_matlab_form(&self, form: &matlab::MatlabForm) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        let request = matlab::lower_request(&mut ms, form);
        self.execute_lowered(&mut ms, request)
    }

    /// Evaluate a held [`WolframForm`] via [`mathematica::lower_request`] on this session.
    pub fn evaluate_wolfram_form(&self, form: &WolframForm) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        let request = mathematica::lower_request(&mut ms, form);
        self.execute_lowered(&mut ms, request)
    }

    /// Re-evaluate an arena term already owned by this session (result objects without Form).
    pub fn evaluate_term_outcome(&self, root: TermId) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        self.execute_lowered(&mut ms, AthenaRequest::Term(root))
    }

    fn execute_lowered(&self, ms: &mut AthenaSession, request: AthenaRequest) -> Result<EvalOutcome, SxoError> {
        // Unify Goal / Term / Control through `execute_request` so status / coverage / diagnostics survive.
        match self.math_engine().execute_request(ms, request) {
            Ok(result_id) => Ok(outcome_from_result(ms, result_id)),
            Err(d) => Err(SxoError::from_diagnostic(d)),
        }
    }

    /// Clear Athena Own symbol definitions for this host session.
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
                materialize_calculus_result_term(&mut dc, &r).unwrap_or(expr)
            }
            _ => self.math_engine().differentiate(&mut ms, expr, var).unwrap_or(expr),
        }
    }

    /// Domain dispatch through Athena.
    pub fn execute_domain(&self, request: DomainRequest) -> Result<DomainResult, Diagnostic> {
        self.math_engine().execute_domain(&mut self.math_session.borrow_mut(), request)
    }

    /// `Simplify` builtin on a term.
    pub fn simplify_term(&self, expr: TermId) -> TermId {
        self.math_engine().simplify(&mut self.math_session.borrow_mut(), expr)
    }

    /// Parse Wolfram text into MMA [`WolframForm`] (no evaluate).
    pub fn parse_mathematica(&self, input: &str) -> Result<WolframForm, SxoError> {
        mathematica::parse_mathematica(input)
    }

    /// MMA form → session arena [`TermId`].
    pub fn lower_mathematica(&self, w: &WolframForm) -> TermId {
        mathematica::lower_wexpr(&mut self.math_session.borrow_mut(), w)
    }

    /// Session arena [`TermId`] → MMA form.
    pub fn to_mathematica(&self, id: TermId) -> WolframForm {
        mathematica::wexpr_from_session(&self.math_session.borrow(), id)
    }

    /// Parse Wolfram, lower via [`mathematica::lower_request`], execute.
    pub fn evaluate_mathematica(&self, input: &str) -> Result<EvalOutcome, SxoError> {
        let w = self.parse_mathematica(input)?;
        self.evaluate_wolfram_form(&w)
    }

    /// Differentiate Wolfram input.
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

    /// Parse MATLAB text into [`matlab::MatlabForm`] (no evaluate).
    pub fn parse_matlab_form(&self, input: &str) -> Result<matlab::MatlabForm, SxoError> {
        matlab::parse_matlab_form(input)
    }

    /// Materialize a MATLAB Form into this session arena.
    pub fn lower_matlab(&self, form: &matlab::MatlabForm) -> TermId {
        matlab::form_to_term(&mut self.math_session.borrow_mut(), form)
    }

    /// Parse MATLAB, lift via Form → Athena request, execute.
    pub fn evaluate_matlab(&self, input: &str) -> Result<EvalOutcome, SxoError> {
        let form = self.parse_matlab_form(input)?;
        self.evaluate_matlab_form(&form)
    }

    /// Parse + dialect Form request path for an explicit dialect tag.
    pub fn evaluate_input(&self, input: &str, dialect: Dialect) -> Result<EvalOutcome, SxoError> {
        match dialect {
            Dialect::Matlab => self.evaluate_matlab(input),
            Dialect::Mathematica => self.evaluate_mathematica(input),
            Dialect::SimpleMath => Err(SxoError::new(
                "simple-math dialect is off the current delivery route (lowercase Form, not Mathematica)",
            )),
        }
    }

    /// Differentiate MATLAB input.
    pub fn d_matlab(&self, input: &str, var: &str) -> Result<TermId, SxoError> {
        Ok(self.differentiate_term(self.parse_matlab(input)?, var))
    }

    /// Render a term as MATLAB text.
    pub fn render_as_matlab(&self, id: TermId) -> String {
        matlab::render_matlab(&self.math_session.borrow(), id)
    }

    /// Try dialect `Plot` / `plot` → SVG via Athena sampling + Apollo.
    ///
    /// Returns `None` when `id` is not a recognized 1-D plot form.
    pub fn try_plot_svg(&self, id: TermId, dialect: Dialect) -> Option<Result<String, SxoError>> {
        let mut ms = self.math_session.borrow_mut();
        match dialect {
            Dialect::Mathematica => mathematica::try_plot_svg(&mut ms, id),
            Dialect::Matlab => matlab::try_plot_svg(&mut ms, id),
            Dialect::SimpleMath => None,
        }
    }

    /// Convenience: indefinite integral via calculus domain.
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

    /// Structural equality within this host session's arena.
    pub fn structural_eq(&self, a: TermId, b: TermId) -> bool {
        self.math_session.borrow().arena.structural_eq(a, b)
    }
}

fn outcome_from_result(ms: &mut AthenaSession, result_id: ResultId) -> EvalOutcome {
    use athena::runtime::values::arena::push_null;

    let Some((status, coverage, diagnostics, symbolic)) = ms.results.get(result_id).map(|result| {
        (
            result.status.name().to_string(),
            result.coverage.name().to_string(),
            result.diagnostics.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
            result.symbolic_term,
        )
    })
    else {
        let null = push_null(ms);
        return EvalOutcome::new(null, ComputationStatus::Unknown.name(), CoverageStatus::Unknown.name(), Vec::new());
    };
    let term = symbolic.unwrap_or_else(|| push_null(ms));
    EvalOutcome::new(term, status, coverage, diagnostics)
}
