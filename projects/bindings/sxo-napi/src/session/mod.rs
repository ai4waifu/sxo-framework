//! Host session: dialect parse/render wired to Athena evaluation.

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session as AthenaSession,
    api::{AthenaRequest, DomainGoal},
    domains::{
        DomainExecutionContext, DomainRequest, DomainResult,
        calculus::{CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder, materialize_calculus_result_term},
        linear_algebra::{ExactDetResult, ExactSolveResult, LinearAlgebraResult, LinearAlgebraValue, MatrixEntry, MatrixValue},
    },
    execution::push_number,
    numeric::{Number, Rational},
    runtime::values::{
        arena::push_list,
        numeric_clone::{clone_integer, clone_rational},
    },
    types::{AssumptionSet, Diagnostic, ResultId, TermId},
};
use sxo_dialect_mathematica::{self as mathematica, WolframForm};
use sxo_dialect_matlab as matlab;
use sxo_types::{Dialect, SxoError};

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
    /// Prefer [`Self::evaluate_form`] for dialect surface that needs `lower_request`.
    pub fn evaluate(&self, expr: TermId) -> TermId {
        self.math_session.borrow_mut().evaluate(expr)
    }

    /// Dialect Form → [`lower_request`] → execute → symbolic term.
    ///
    /// Prefer [`Self::evaluate_input`] / [`Self::evaluate_matlab`] for source text.
    /// This `TermId` entry is transitional: MATLAB uses [`matlab::lower_term_request`]
    /// and must not grow new request-shaped heads (add those on `MatlabForm` instead).
    pub fn evaluate_form(&self, root: TermId, dialect: Dialect) -> Result<TermId, SxoError> {
        match dialect {
            Dialect::Matlab => {
                let mut ms = self.math_session.borrow_mut();
                let request = matlab::lower_term_request(&mut ms, root);
                self.execute_lowered(&mut ms, request, root)
            }
            Dialect::Mathematica | Dialect::SimpleMath => {
                let w = self.to_mathematica(root);
                let mut ms = self.math_session.borrow_mut();
                let request = mathematica::lower_request(&mut ms, &w);
                let fallback = mathematica::lower_wexpr(&mut ms, &w);
                self.execute_lowered(&mut ms, request, fallback)
            }
        }
    }

    fn execute_lowered(&self, ms: &mut AthenaSession, request: AthenaRequest, fallback: TermId) -> Result<TermId, SxoError> {
        match request {
            AthenaRequest::Goal(DomainGoal::Dispatch(domain)) => match self.math_engine().execute_domain(ms, domain) {
                Ok(domain) => Ok(materialize_domain_term(ms, domain).unwrap_or(fallback)),
                Err(d) => Err(SxoError::from_diagnostic(d)),
            },
            other => match self.math_engine().execute_request(ms, other) {
                Ok(result_id) => Ok(symbolic_or_fallback(ms, result_id, fallback)),
                Err(d) => Err(SxoError::from_diagnostic(d)),
            },
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
                materialize_calculus_result_term(&mut dc, &r)
            }
            _ => self.math_engine().differentiate(&mut ms, expr, var),
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
    pub fn evaluate_mathematica(&self, input: &str) -> Result<TermId, SxoError> {
        let w = self.parse_mathematica(input)?;
        let root = self.lower_mathematica(&w);
        self.evaluate_form(root, Dialect::Mathematica)
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

    /// Parse MATLAB, lift via Form → Athena request, execute.
    pub fn evaluate_matlab(&self, input: &str) -> Result<TermId, SxoError> {
        let form = matlab::parse_matlab_form(input)?;
        let mut ms = self.math_session.borrow_mut();
        let request = matlab::lower_request(&mut ms, &form);
        let fallback = match &request {
            AthenaRequest::Term(term) => *term,
            _ => matlab::form_to_term(&mut ms, &form),
        };
        self.execute_lowered(&mut ms, request, fallback)
    }

    /// Parse + dialect Form request path for an explicit dialect tag.
    pub fn evaluate_input(&self, input: &str, dialect: Dialect) -> Result<TermId, SxoError> {
        match dialect {
            Dialect::Matlab => self.evaluate_matlab(input),
            Dialect::Mathematica | Dialect::SimpleMath => {
                let w = self.parse_mathematica(input)?;
                let mut ms = self.math_session.borrow_mut();
                let request = mathematica::lower_request(&mut ms, &w);
                let fallback = mathematica::lower_wexpr(&mut ms, &w);
                self.execute_lowered(&mut ms, request, fallback)
            }
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

fn symbolic_or_fallback(ms: &AthenaSession, result_id: ResultId, fallback: TermId) -> TermId {
    ms.results.get(result_id).and_then(|r| r.symbolic_term).unwrap_or(fallback)
}

fn materialize_domain_term(ms: &mut AthenaSession, domain: DomainResult) -> Option<TermId> {
    match domain {
        DomainResult::Calculus(r) => {
            let mut dc = DomainExecutionContext::new(ms);
            Some(materialize_calculus_result_term(&mut dc, &r))
        }
        DomainResult::LinearAlgebra(LinearAlgebraResult::Ok { value }) => match value {
            LinearAlgebraValue::Matrix(m) => matrix_to_nested_list(ms, &m).ok(),
            LinearAlgebraValue::ExactSolve(ExactSolveResult { particular: Some(m), .. }) => matrix_to_nested_list(ms, &m).ok(),
            LinearAlgebraValue::ExactDet(ExactDetResult { det, .. }) => Some(rational_to_term(ms, &det)),
            _ => None,
        },
        _ => None,
    }
}

fn rational_to_term(session: &mut AthenaSession, r: &Rational) -> TermId {
    if r.is_integer() {
        if let Some(i) = r.numerator().to_i64() {
            return session.builder().int(i, Default::default());
        }
    }
    push_number(session, Number::from_rational_normalized(clone_rational(r)))
}

fn matrix_to_nested_list(session: &mut AthenaSession, m: &MatrixValue) -> Result<TermId, Diagnostic> {
    let (rows, cols) = (m.shape().rows, m.shape().cols);
    let mut out = Vec::with_capacity(rows as usize);
    for i in 0..rows {
        let mut row = Vec::with_capacity(cols as usize);
        for j in 0..cols {
            match m.get(i, j)? {
                MatrixEntry::Rational(r) => row.push(rational_to_term(session, &r)),
                MatrixEntry::Integer(n) => {
                    if let Some(i64v) = n.to_i64() {
                        row.push(session.builder().int(i64v, Default::default()));
                    }
                    else {
                        row.push(push_number(session, Number::integer(clone_integer(&n))));
                    }
                }
                MatrixEntry::MachineF64(x) => row.push(push_number(session, Number::machine(x))),
            }
        }
        out.push(push_list(session, row));
    }
    Ok(push_list(session, out))
}
