//! Host session: dialect parse/render wired to Athena evaluation.

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session as AthenaSession,
    api::{AthenaRequest, DomainGoal},
    domains::{
        DomainRequest, DomainResult,
        calculus::{CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder},
    },
    ir::SemanticOperator,
    types::{AssumptionSet, Diagnostic, ResultId, TermId},
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

    /// Evaluate a term through Athena (no Athena-term reverse-parse into calculus Goal).
    /// Prefer Form evaluate helpers for dialect surface that needs `lower_request`.
    #[allow(dead_code)]
    pub fn evaluate(&self, expr: TermId) -> Result<TermId, SxoError> {
        self.math_session.borrow_mut().evaluate(expr).map_err(SxoError::from_diagnostic)
    }

    /// Evaluate a held MATLAB Form via [`matlab::lower_request`] on this session.
    pub fn evaluate_matlab_form(&self, form: &matlab::MatlabForm) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        matlab::with_session_conventions(&mut ms, |ms| {
            let request = matlab::lower_request(ms, form);
            self.execute_lowered(ms, request)
        })
    }

    /// Evaluate a held Wolfram Form via [`mathematica::lower_request`] on this session.
    pub fn evaluate_wolfram_form(&self, form: &WolframForm) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        let request = mathematica::lower_request(&mut ms, form);
        self.execute_lowered(&mut ms, request)
    }

    /// Re-evaluate an arena term already owned by this session.
    pub fn evaluate_term_outcome(&self, root: TermId) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        self.execute_lowered(&mut ms, athena::api::AthenaRequest::Term(root))
    }

    fn execute_lowered(&self, ms: &mut AthenaSession, request: athena::api::AthenaRequest) -> Result<EvalOutcome, SxoError> {
        match self.math_engine().execute_request(ms, request) {
            Ok(result_id) => Ok(outcome_from_result(ms, result_id)),
            Err(d) => Err(SxoError::from_diagnostic(d)),
        }
    }

    /// Project a Session-local [`ResultId`] to a symbolic [`TermId`].
    ///
    /// Errors when missing or when the result has no symbolic projection. Does not invent `Null`.
    pub fn try_project_symbolic(&self, result_id: ResultId) -> Result<TermId, SxoError> {
        let ms = self.math_session.borrow();
        match ms.results.get(result_id) {
            None => Err(SxoError::new("unknown ResultId")),
            Some(result) => match result.symbolic_term {
                Some(term) => Ok(term),
                None => Err(SxoError::new(format!(
                    "result has no symbolic Term projection (status={}, coverage={})",
                    result.status.name(),
                    result.coverage.name()
                ))),
            },
        }
    }

    /// Project a Session-local [`ResultId`] to a symbolic [`TermId`].
    #[allow(dead_code)]
    pub fn project_result(&self, result_id: ResultId) -> Result<TermId, SxoError> {
        self.try_project_symbolic(result_id)
    }

    /// Project diagnostic summaries for a Session-local [`ResultId`] (empty if missing).
    pub fn project_diagnostics(&self, result_id: ResultId) -> Vec<String> {
        let ms = self.math_session.borrow();
        ms.results.get(result_id).map(|r| r.diagnostics.iter().map(|d| d.to_string()).collect()).unwrap_or_default()
    }

    /// Project condition summaries for a Session-local [`ResultId`] (empty if missing).
    pub fn project_conditions(&self, result_id: ResultId) -> Vec<String> {
        let ms = self.math_session.borrow();
        ms.results
            .get(result_id)
            .map(|r| r.conditions.iter().map(sxo_types::condition_summary).collect())
            .unwrap_or_default()
    }

    /// Project provider stamp summary for a Session-local [`ResultId`] (`None` if missing).
    pub fn project_provider(&self, result_id: ResultId) -> Option<String> {
        let ms = self.math_session.borrow();
        ms.results
            .get(result_id)
            .and_then(|r| r.provider.as_ref())
            .map(|stamp| format!("{}@v{}", stamp.id.name(), stamp.version))
    }

    /// Project evidence summaries for a Session-local [`ResultId`] (empty if missing).
    pub fn project_evidence(&self, result_id: ResultId) -> Vec<String> {
        use athena::runtime::results::ResultEvidence;
        let ms = self.math_session.borrow();
        ms.results
            .get(result_id)
            .map(|r| {
                r.evidence
                    .iter()
                    .map(|e| match e {
                        ResultEvidence::TrustedKernelSummary { provider, summary } => {
                            sxo_types::trusted_kernel_evidence_summary(provider.name(), summary)
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clear Athena Own symbol definitions for this host session.
    #[allow(dead_code)]
    pub fn clear_definitions(&self) {
        self.math_session.borrow_mut().clear_definitions();
    }

    /// Differentiate via [`AthenaRequest::Goal`] and return the full Session result.
    ///
    /// Prefer Form helpers for parse objects. This path remains for result term projection.
    pub fn differentiate_outcome(&self, expr: TermId, var: &str) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        let variable = ms.arena.symbols_mut().intern(var);
        let request = AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::Calculus(CalculusRequest::Derivative {
            expression: expr,
            variable,
            order: DerivativeOrder::First,
            assumptions: AssumptionSet::empty(),
        })));
        self.execute_lowered(&mut ms, request)
    }

    /// Differentiate a retained Wolfram Form via `D[expr, var]` → dialect `lower_request` Goal path.
    pub fn differentiate_wolfram_form(&self, form: &WolframForm, var: &str) -> Result<EvalOutcome, SxoError> {
        let d_form = WolframForm::call("D", vec![form.clone(), WolframForm::symbol(var)]);
        self.evaluate_wolfram_form(&d_form)
    }

    /// Differentiate a retained MATLAB Form via `diff(expr, var)` → dialect `lower_request` Goal path.
    pub fn differentiate_matlab_form(&self, form: &matlab::MatlabForm, var: &str) -> Result<EvalOutcome, SxoError> {
        let d_form = matlab::MatlabForm::call("diff", vec![form.clone(), matlab::MatlabForm::symbol(var)]);
        self.evaluate_matlab_form(&d_form)
    }

    /// Differentiate and project a symbolic term.
    pub fn differentiate_term(&self, expr: TermId, var: &str) -> Result<TermId, SxoError> {
        let outcome = self.differentiate_outcome(expr, var)?;
        self.try_project_symbolic(outcome.result_id)
    }

    /// Domain dispatch through Athena.
    #[allow(dead_code)]
    pub fn execute_domain(&self, request: DomainRequest) -> Result<DomainResult, Diagnostic> {
        self.math_engine().execute_domain(&mut self.math_session.borrow_mut(), request)
    }

    /// `Simplify` via [`AthenaRequest::Term`] and return the full Session result.
    ///
    /// Prefer Form helpers for parse objects. This path remains for result term projection.
    pub fn simplify_outcome(&self, expr: TermId) -> Result<EvalOutcome, SxoError> {
        let mut ms = self.math_session.borrow_mut();
        let wrapped = athena::execution::push_semantic(&mut ms, SemanticOperator::Simplify, vec![expr]);
        self.execute_lowered(&mut ms, AthenaRequest::Term(wrapped))
    }

    /// Simplify a retained Wolfram Form via `Simplify[expr]` → dialect `lower_request`.
    pub fn simplify_wolfram_form(&self, form: &WolframForm) -> Result<EvalOutcome, SxoError> {
        let s_form = WolframForm::call("Simplify", vec![form.clone()]);
        self.evaluate_wolfram_form(&s_form)
    }

    /// Simplify a retained MATLAB Form via `Simplify(expr)` → dialect `lower_request`.
    pub fn simplify_matlab_form(&self, form: &matlab::MatlabForm) -> Result<EvalOutcome, SxoError> {
        let s_form = matlab::MatlabForm::call("Simplify", vec![form.clone()]);
        self.evaluate_matlab_form(&s_form)
    }

    /// Record that `child` was derived from `parent` (both Session-local).
    pub fn link_derived_from(&self, child: ResultId, parent: ResultId) -> bool {
        self.math_session.borrow_mut().results.link_derived_from(child, parent)
    }

    /// Parent [`ResultId`] recorded on `id`, if any.
    pub fn derived_from(&self, id: ResultId) -> Option<ResultId> {
        self.math_session.borrow().results.get(id).and_then(|r| r.derived_from)
    }

    /// `Simplify` and project a symbolic term.
    #[allow(dead_code)]
    pub fn simplify_term(&self, expr: TermId) -> Result<TermId, SxoError> {
        let outcome = self.simplify_outcome(expr)?;
        self.try_project_symbolic(outcome.result_id)
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
    #[allow(dead_code)]
    pub fn evaluate_mathematica(&self, input: &str) -> Result<EvalOutcome, SxoError> {
        let w = self.parse_mathematica(input)?;
        self.evaluate_wolfram_form(&w)
    }

    /// Differentiate Wolfram input via Form `D[…]` request path.
    #[allow(dead_code)]
    pub fn d_mathematica(&self, input: &str, var: &str) -> Result<TermId, SxoError> {
        let w = self.parse_mathematica(input)?;
        let outcome = self.differentiate_wolfram_form(&w, var)?;
        self.try_project_symbolic(outcome.result_id)
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
    #[allow(dead_code)]
    pub fn evaluate_matlab(&self, input: &str) -> Result<EvalOutcome, SxoError> {
        let form = self.parse_matlab_form(input)?;
        self.evaluate_matlab_form(&form)
    }

    /// Parse + dialect Form request path for an explicit dialect tag.
    pub fn evaluate_input(&self, input: &str, dialect: Dialect) -> Result<EvalOutcome, SxoError> {
        match dialect {
            Dialect::Matlab => self.evaluate_matlab(input),
            Dialect::Mathematica => self.evaluate_mathematica(input),
            Dialect::SimpleMath => {
                Err(SxoError::new("simple-math dialect is off the current delivery route (lowercase Form, not Mathematica)"))
            }
        }
    }

    /// Differentiate MATLAB input via Form `diff(…)` request path.
    #[allow(dead_code)]
    pub fn d_matlab(&self, input: &str, var: &str) -> Result<TermId, SxoError> {
        let form = self.parse_matlab_form(input)?;
        let outcome = self.differentiate_matlab_form(&form, var)?;
        self.try_project_symbolic(outcome.result_id)
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
            Dialect::SimpleMath => None,
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

fn outcome_from_result(ms: &AthenaSession, result_id: ResultId) -> EvalOutcome {
    use athena::{runtime::CoverageStatus, types::ComputationStatus};

    let Some(result) = ms.results.get(result_id)
    else {
        return EvalOutcome::new(result_id, ComputationStatus::Unknown.name(), CoverageStatus::Unknown.name());
    };
    EvalOutcome::new(result_id, result.status.name().to_string(), result.coverage.name().to_string())
}
