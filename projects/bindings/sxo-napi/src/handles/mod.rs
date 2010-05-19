//! Opaque N-API expression handles.

use std::rc::Rc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use sxo_types::Dialect;

use crate::{
    dialects::{HeldForm, dialect_from_str, dialect_to_str, map_err, parse_held, render_held},
    session::Session,
};
use athena::types::{ResultId, TermId};

/// Opaque expression handle backed by a shared host [`Session`].
///
/// Parse objects retain a dialect [`HeldForm`] and do not materialize arena terms until
/// evaluate / `d` / `simplify` / plot needs them. Display uses Form renderers when present.
/// Evaluate results retain a Session-local [`ResultId`] and project [`TermId`] on demand.
#[derive(Debug)]
#[napi]
pub struct Expression {
    pub(crate) session: Rc<Session>,
    /// Present after `d` / `simplify` (or string-entry helpers that materialize a bare term).
    pub(crate) root: Option<TermId>,
    /// Present after evaluate. Prefer over `root` when projecting symbolic results.
    pub(crate) result_id: Option<ResultId>,
    /// Present on parse objects. Cleared on evaluate / `d` / `simplify` results.
    pub(crate) form: Option<HeldForm>,
    pub(crate) dialect: Dialect,
    /// Athena [`ComputationStatus`] name from the last evaluate (or `Unknown` if not evaluated).
    pub(crate) status: String,
    /// Coverage name from the last evaluate (or `Unknown` if not evaluated).
    pub(crate) coverage: String,
    /// Diagnostic summaries from the last evaluate.
    pub(crate) diagnostics: Vec<String>,
}

fn with_outcome(session: Rc<Session>, dialect: Dialect, outcome: sxo_types::EvalOutcome) -> Expression {
    Expression {
        session,
        root: None,
        result_id: Some(outcome.result_id),
        form: None,
        dialect,
        status: outcome.status,
        coverage: outcome.coverage,
        diagnostics: outcome.diagnostics,
    }
}

fn unevaluated_form(session: Rc<Session>, form: HeldForm, dialect: Dialect) -> Expression {
    Expression {
        session,
        root: None,
        result_id: None,
        form: Some(form),
        dialect,
        status: "Unknown".into(),
        coverage: "Unknown".into(),
        diagnostics: Vec::new(),
    }
}

fn unevaluated_term(session: Rc<Session>, root: TermId, dialect: Dialect) -> Expression {
    Expression {
        session,
        root: Some(root),
        result_id: None,
        form: None,
        dialect,
        status: "Unknown".into(),
        coverage: "Unknown".into(),
        diagnostics: Vec::new(),
    }
}

impl Expression {
    /// Materialize a Term once when an API still requires [`TermId`].
    fn materialize_root(&self) -> Result<TermId> {
        if let Some(result_id) = self.result_id {
            return Ok(self.session.project_result(result_id));
        }
        if let Some(root) = self.root {
            return Ok(root);
        }
        match &self.form {
            Some(HeldForm::Matlab(form)) => Ok(self.session.lower_matlab(form)),
            Some(HeldForm::Wolfram(form)) => Ok(self.session.lower_mathematica(form)),
            None => Err(Error::from_reason("expression has neither Form, ResultId, nor Term root")),
        }
    }
}

#[napi]
impl Expression {
    /// Parse `input` with an explicit dialect (`mathematica` | `matlab` | `simple-math`).
    #[napi(factory)]
    pub fn parse(input: String, dialect: Option<String>) -> Result<Self> {
        let d = dialect_from_str(dialect)?;
        let session = Rc::new(Session::new());
        let (form, resolved) = parse_held(&session, &input, d)?;
        Ok(unevaluated_form(session, form, resolved))
    }

    /// Differentiate with respect to `var` on the same session.
    #[napi]
    pub fn d(&self, var: String) -> Result<Expression> {
        let term = self.materialize_root()?;
        let root = self.session.differentiate_term(term, &var);
        Ok(unevaluated_term(Rc::clone(&self.session), root, self.dialect))
    }

    /// Simplify via the engine (`Simplify` head) on the same session.
    #[napi]
    pub fn simplify(&self) -> Result<Expression> {
        let term = self.materialize_root()?;
        let root = self.session.simplify_term(term);
        Ok(unevaluated_term(Rc::clone(&self.session), root, self.dialect))
    }

    /// Evaluate from retained Form (parse objects) or arena term (result objects).
    #[napi]
    pub fn evaluate(&self) -> Result<Expression> {
        let outcome = match &self.form {
            Some(HeldForm::Matlab(form)) => self.session.evaluate_matlab_form(form),
            Some(HeldForm::Wolfram(form)) => self.session.evaluate_wolfram_form(form),
            None => {
                let root = self.materialize_root()?;
                self.session.evaluate_term_outcome(root)
            }
        }
        .map_err(map_err)?;
        Ok(with_outcome(Rc::clone(&self.session), self.dialect, outcome))
    }

    /// Athena computation status name (`Exact`, `Candidate`, `Unknown`, …).
    #[napi(getter)]
    pub fn status(&self) -> String {
        self.status.clone()
    }

    /// Coverage name (`Full`, `Partial`, `Unknown`, `Unsupported`).
    #[napi(getter)]
    pub fn coverage(&self) -> String {
        self.coverage.clone()
    }

    /// Diagnostic summaries from the last evaluate (empty if none / not evaluated).
    #[napi(getter)]
    pub fn diagnostics(&self) -> Vec<String> {
        self.diagnostics.clone()
    }

    /// Render as string in the expression's dialect.
    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> Result<String> {
        if let Some(form) = &self.form {
            return Ok(render_held(form));
        }
        let root = self.materialize_root()?;
        Ok(match self.dialect {
            Dialect::Matlab => self.session.render_as_matlab(root),
            _ => self.session.render_as_wolfram(root),
        })
    }

    /// Render as Mathematica / Wolfram text.
    #[napi(js_name = "toWolfram")]
    pub fn to_wolfram(&self) -> Result<String> {
        if let Some(HeldForm::Wolfram(w)) = &self.form {
            return Ok(sxo_dialect_mathematica::render(w));
        }
        let root = self.materialize_root()?;
        Ok(self.session.render_as_wolfram(root))
    }

    /// Render as MATLAB text.
    #[napi(js_name = "toMatlab")]
    pub fn to_matlab(&self) -> Result<String> {
        if let Some(HeldForm::Matlab(f)) = &self.form {
            return Ok(sxo_dialect_matlab::render_matlab_form(f));
        }
        let root = self.materialize_root()?;
        Ok(self.session.render_as_matlab(root))
    }

    /// Structural equality (Form compare when both held; else Term via Mathematica projection).
    #[napi(js_name = "isEqual")]
    pub fn is_equal(&self, other: &Expression) -> Result<bool> {
        match (&self.form, &other.form) {
            (Some(HeldForm::Matlab(a)), Some(HeldForm::Matlab(b))) => Ok(a == b),
            (Some(HeldForm::Wolfram(a)), Some(HeldForm::Wolfram(b))) => Ok(a == b),
            _ => {
                let a = self.materialize_root()?;
                let b = other.materialize_root()?;
                Ok(self.session.to_mathematica(a) == other.session.to_mathematica(b))
            }
        }
    }

    /// Render 1-D `Plot` / `plot` as SVG when the term matches a known form.
    #[napi(js_name = "plotSvg")]
    pub fn plot_svg(&self) -> Result<String> {
        let root = self.materialize_root()?;
        match self.session.try_plot_svg(root, self.dialect) {
            Some(Ok(svg)) => Ok(svg),
            Some(Err(e)) => Err(map_err(e)),
            None => Err(Error::from_reason("not a supported 1-D plot form")),
        }
    }

    /// Dialect tag used for default rendering.
    #[napi(getter)]
    pub fn dialect(&self) -> String {
        dialect_to_str(self.dialect).to_string()
    }
}
