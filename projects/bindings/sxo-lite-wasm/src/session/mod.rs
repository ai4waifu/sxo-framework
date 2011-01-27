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
        ms.results.get(result_id).map(|r| r.conditions.iter().map(sxo_types::condition_summary).collect()).unwrap_or_default()
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
                        ResultEvidence::AdmittedRelation { fact } => sxo_types::admitted_relation_evidence_summary(fact.0),
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

    /// Differentiate a retained result in one outcome.
    ///
    /// Projects a symbolic term, dispatches one calculus Goal, and records `derived_from`.
    /// A missing projection is an error. This does not invent `Null`.
    pub fn differentiate_result(&self, parent: ResultId, var: &str) -> Result<EvalOutcome, SxoError> {
        let term = self.try_project_symbolic(parent)?;
        let outcome = self.differentiate_outcome(term, var)?;
        let _ = self.link_derived_from(outcome.result_id, parent);
        Ok(outcome)
    }

    /// Simplify a retained result in one outcome.
    ///
    /// Projects a symbolic term, dispatches one Simplify, and records `derived_from`.
    /// A missing projection is an error. This does not invent `Null`.
    pub fn simplify_result(&self, parent: ResultId) -> Result<EvalOutcome, SxoError> {
        let term = self.try_project_symbolic(parent)?;
        let outcome = self.simplify_outcome(term)?;
        let _ = self.link_derived_from(outcome.result_id, parent);
        Ok(outcome)
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

#[cfg(test)]
mod parity_tests {
    use super::Session;

    /// Same Session contract the WASM ABI calls: direct string vs parse→Form→evaluate.
    #[test]
    fn matlab_direct_and_handle_matrix_parity() {
        let cases = [
            ("(1+2)*3", "9"),
            ("[1, 2; 3, 4].'", "[1, 3; 2, 4]"),
            ("[1, 2; 3, 4]'", "[1, 3; 2, 4]"),
        ("[1+2i, 3; 4, 5]'", "[1 - 2*i, 4; 3, 5]"),
            ("[1+i, 0; 0, 1-i]*[1, i; -i, 1]", "[1 + i, -1 + i; -1 - i, 1 - i]"),
            ("[1+i, 2; 3, 4].'", "[1 + i, 3; 2, 4]"),
            ("tril([1+i, 2; 3, 4-i])", "[1 + i, 0; 3, 4 - i]"),
            ("triu([1+i, 2; 3, 4-i])", "[1 + i, 2; 0, 4 - i]"),
            ("[1+i, 2; 3, 4](1,:)", "[1 + i, 2]"),
            ("[1+i, 2; 3, 4](:,2)", "[2, 4]"),
            ("[1+i, 2; 3, 4](1,2)", "2"),
            ("kron([1, 2], [3, 4])", "[3, 4, 6, 8]"),
            ("kron([1+i], [1, i])", "[1 + i, -1 + i]"),
            ("[1+i].^2", "2*i"),
            ("M=[1, 2; 3, 4]; M(:, 2)=[9; 8]; M", "[1, 9; 3, 8]"),
            ("M=[1, 2; 3, 4]; M(1, :)=[9, 8]; M", "[9, 8; 3, 4]"),
            ("A=zeros(2); A(3, 3)=1; A", "[0, 0, 0; 0, 0, 0; 0, 0, 1]"),
            ("B=1:4; B(end+1)=5; B", "[1, 2, 3, 4, 5]"),
            ("A=[1, 2; 3, 4]; A(2)", "3"),
            ("A=[1, 2; 3, 4]; A(1, 2)", "2"),
            ("[1, 2; 3, 4](:)", "[1; 3; 2; 4]"),
            ("[1, 2; 3, 4].*[5, 6; 7, 8]", "[5, 12; 21, 32]"),
            ("[1+i, 2; 3, 4].*[1, i; 0, 1]", "[1 + i, 2*i; 0, 4]"),
            ("[1, 2; 3, 4]*[5, 6; 7, 8]", "[19, 22; 43, 50]"),
            ("[6, 8; 10, 12]./[2, 4; 5, 6]", "[3, 2; 2, 2]"),
            ("[1, 2; 3, 4]/[1, 2; 3, 4]", "[1, 0; 0, 1]"),
            ("inv([1, 2; 2, 4])", "inv(Singular)"),
            ("cond([1, 2; 2, 4])", "inf"),
            ("[1, 2; 3, 4] \\ [5; 6]", "[-4; 9/2]"),
            ("[1, 2; 2, 4] \\ [1; 0]", "linsolve(Inconsistent)"),
            ("[1, 2; 2, 4] \\ [2; 4]", "linsolve(Infinite, 1)"),
            ("[1, 0] / [1, 2; 2, 4]", "linsolve(Inconsistent)"),
            ("[1, 2; 2, 4] / [1, 2; 2, 4]", "linsolve(Infinite, 1)"),
            ("0/0", "NaN"),
            ("(1/0)-(1/0)", "NaN"),
            ("Inf - Inf", "NaN"),
            ("0.0/0.0", "NaN"),
            ("sum([1, 2, 3])", "6"),
            ("sum([1, 2; 3, 4])", "[4, 6]"),
            ("sum([1, 2; 3, 4], 2)", "[3; 7]"),
            ("prod([2, 3, 4])", "24"),
            ("prod([1, 2; 3, 4])", "[3, 8]"),
            ("sum([1+i, 2; 3, 4-i])", "[4 + i, 6 - i]"),
            ("sum([1+i, 2; 3, 4-i], 2)", "[3 + i; 7 - i]"),
            ("prod([1+i, 2; 3, 4-i], 2)", "[2 + 2*i; 12 - 3*i]"),
            ("prod([1+i, 2; 3, 4-i])", "[3 + 3*i, 8 - 2*i]"),
            ("cumsum([1+i, 2, 3])", "[1 + i, 3 + i, 6 + i]"),
            ("diag([1+i, 2])", "[1 + i, 0; 0, 2]"),
            ("[1, 2, 3](end)", "3"),
        ];
        for (input, expected) in cases {
            let direct_session = Session::new();
            let direct = direct_session.evaluate_matlab(input).unwrap();
            let direct_text = direct_session.render_as_matlab(direct_session.project_result(direct.result_id).unwrap());

            let handle_session = Session::new();
            let form = handle_session.parse_matlab_form(input).unwrap();
            let via_handle = handle_session.evaluate_matlab_form(&form).unwrap();
            let handle_text = handle_session.render_as_matlab(handle_session.project_result(via_handle.result_id).unwrap());

            assert_eq!(direct_text, handle_text, "parity failed for {input}: direct={direct_text} handle={handle_text}");
            assert_eq!(direct_text, expected, "expected value for {input}");
            assert_eq!(direct.status, via_handle.status, "status parity for {input}");
            assert_eq!(direct.coverage, via_handle.coverage, "coverage parity for {input}");
            assert_ne!(direct_text, "Null", "projection must not invent Null for {input}");
        }
    }

    /// Same Session contract for MMA matrix surfaces.
    #[test]
    fn mathematica_direct_and_handle_matrix_parity() {
        let cases = [
            ("Transpose[{{1, 2}, {3, 4}}]", "{{1, 3}, {2, 4}}"),
            ("ConjugateTranspose[{{1, 2}, {3, 4}}]", "{{1, 3}, {2, 4}}"),
        ("ConjugateTranspose[{{1 + 2 I, 3}, {4, 5}}]", "{{1 - 2*I, 4}, {3, 5}}"),
            ("Dot[{{1 + I, 0}, {0, 1 - I}}, {{1, I}, {-I, 1}}]", "{{1 + I, -1 + I}, {-1 - I, 1 - I}}"),
            ("Transpose[{{1 + I, 2}, {3, 4}}]", "{{1 + I, 3}, {2, 4}}"),
            ("LowerTriangularize[{{1 + I, 2}, {3, 4 - I}}]", "{{1 + I, 0}, {3, 4 - I}}"),
            ("UpperTriangularize[{{1 + I, 2}, {3, 4 - I}}]", "{{1 + I, 2}, {0, 4 - I}}"),
            ("Reverse[{{1 + I, 2}, {3, 4}}]", "{{3, 4}, {1 + I, 2}}"),
            ("KroneckerProduct[{1, 2}, {3, 4}]", "{3, 4, 6, 8}"),
            ("KroneckerProduct[{{1 + I}}, {{1, I}}]", "{1 + I, -1 + I}"),
            ("{{1 + I}}^2", "2*I"),
            ("A={{1, 2}, {3, 4}}; ReplacePart[A, {1, 2} -> 9]; A", "{{1, 9}, {3, 4}}"),
            ("SymmetricMatrixQ[{{1, 2}, {2, 1}}]", "1"),
            ("SymmetricMatrixQ[{{1, 2}, {3, 4}}]", "0"),
            ("Inverse[{{1, 2}, {3, 4}}]", "{{-2, 1}, {3/2, -1/2}}"),
            ("Part[{{1, 2}, {3, 4}}, 1, 2]", "2"),
            ("Part[{{1 + I, 2}, {3, 4}}, 1, 2]", "2"),
            ("0/0", "Indeterminate"),
            ("(1/0)-(1/0)", "Indeterminate"),
            ("ReplacePart[{1, 2, 3}, 2 -> 9]", "{1, 9, 3}"),
            ("Infinity - Infinity", "Indeterminate"),
            ("0.0/0.0", "Indeterminate"),
            ("Dot[{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}]", "{{19, 22}, {43, 50}}"),
            ("{{1, 2}, {3, 4}}*{{5, 6}, {7, 8}}", "{{5, 12}, {21, 32}}"),
            ("{{1 + I, 2}, {3, 4}}*{{1, I}, {0, 1}}", "{{1 + I, 2*I}, {0, 4}}"),
            ("LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]", "{{-4}, {9/2}}"),
            ("LinearSolve[{{1, 2}, {2, 4}}, {{1}, {0}}]", "LinearSolve[Inconsistent]"),
            ("LinearSolve[{{1, 2}, {2, 4}}, {{2}, {4}}]", "LinearSolve[Infinite, 1]"),
            ("Total[{1, 2, 3}]", "6"),
            ("Total[{{1, 2}, {3, 4}}]", "{4, 6}"),
            ("Total[{{1 + I, 2}, {3, 4 - I}}]", "{4 + I, 6 - I}"),
            ("Total[{{1, 2}, {3, 4}}, {2}]", "{{3}, {7}}"),
            ("Total[{{1 + I, 2}, {3, 4 - I}}, {2}]", "{{3 + I}, {7 - I}}"),
            ("Product[{{1 + I, 2}, {3, 4 - I}}]", "{3 + 3*I, 8 - 2*I}"),
            ("Product[{{1 + I, 2}, {3, 4 - I}}, {2}]", "{{2 + 2*I}, {12 - 3*I}}"),
            ("Partition[{1, 2, 3, 4}, 2]", "{{1, 2}, {3, 4}}"),
            ("Accumulate[{1 + I, 2, 3}]", "{1 + I, 3 + I, 6 + I}"),
            ("Differences[Range[4]]", "{1, 1, 1}"),
            ("Differences[{1 + I, 2, 3 - I}]", "{1 - I, 1 - I}"),
            ("DiagonalMatrix[{1 + I, 2}]", "{{1 + I, 0}, {0, 2}}"),
            ("ConstantArray[I, 3]", "{I, I, I}"),
            ("Append[{1, 2}, I]", "{1, 2, I}"),
            ("Prepend[{1, 2}, I]", "{I, 1, 2}"),
            ("PadLeft[{1 + I, 2}, 4]", "{0, 0, 1 + I, 2}"),
            ("Reverse[{1 + I, 2, 3}]", "{3, 2, 1 + I}"),
            ("Take[{1 + I, 2, 3, 4}, 2]", "{1 + I, 2}"),
            ("First[{1 + I, 2, 3}]", "1 + I"),
            ("Extract[{1 + I, 2, 3}, 2]", "2"),
            (
                "Flatten[{{1 + I, 2}, {3, 4 - I}}]",
                "{1 + I, 2, 3, 4 - I}",
            ),
            ("Join[{{1 + I}}, {{2}}]", "{{1 + I}, {2}}"),
            ("Join[{{1 + I, 2}}, {{3, 4}}]", "{{1 + I, 2}, {3, 4}}"),
            ("Riffle[{1, 2}, {I, 3}]", "{1, I, 2, 3}"),
            ("Transpose[{{1, 2}, {3}}]", "Transpose[{{1, 2}, {3}}]"),
            ("Inverse[{{1, 2}, {2, 4}}]", "Inverse[Singular]"),
            ("MapAt[f, {1, 2, 3}, 2]", "MapAt[f, {1, 2, 3}, 2]"),
        ];
        for (input, expected) in cases {
            let direct_session = Session::new();
            let direct = direct_session.evaluate_mathematica(input).unwrap();
            let direct_text = direct_session.render_as_wolfram(direct_session.project_result(direct.result_id).unwrap());

            let handle_session = Session::new();
            let form = handle_session.parse_mathematica(input).unwrap();
            let via_handle = handle_session.evaluate_wolfram_form(&form).unwrap();
            let handle_text = handle_session.render_as_wolfram(handle_session.project_result(via_handle.result_id).unwrap());

            assert_eq!(direct_text, handle_text, "parity failed for {input}: direct={direct_text} handle={handle_text}");
            assert_eq!(direct_text, expected, "expected value for {input}");
            assert_eq!(direct.status, via_handle.status, "status parity for {input}");
            assert_eq!(direct.coverage, via_handle.coverage, "coverage parity for {input}");
            assert_ne!(direct_text, "Null", "projection must not invent Null for {input}");
        }
    }

    #[test]
    fn result_transforms_record_derived_from_on_the_same_outcome() {
        let session = Session::new();
        let evaluated = session.evaluate_mathematica("x^3").unwrap();
        let derived = session.differentiate_result(evaluated.result_id, "x").unwrap();
        assert_eq!(session.derived_from(derived.result_id), Some(evaluated.result_id));
        assert!(!derived.status.is_empty());
        assert!(!derived.coverage.is_empty());
        let via_surface = session.evaluate_mathematica("D[x^3, x]").unwrap();
        assert_eq!(
            session.render_as_wolfram(session.project_result(derived.result_id).unwrap()),
            session.render_as_wolfram(session.project_result(via_surface.result_id).unwrap())
        );
        assert_ne!(session.render_as_wolfram(session.project_result(derived.result_id).unwrap()), "Null");

        let sum = session.evaluate_matlab("sin(x)^2 + cos(x)^2").unwrap();
        let simplified = session.simplify_result(sum.result_id).unwrap();
        assert_eq!(session.derived_from(simplified.result_id), Some(sum.result_id));
        assert_ne!(simplified.result_id, sum.result_id, "simplify must publish a new ResultId");
        assert_eq!(session.render_as_matlab(session.project_result(simplified.result_id).unwrap()), "1");
        assert!(!simplified.status.is_empty());
        assert!(!simplified.coverage.is_empty());
    }
}
