//! Opaque N-API expression handles.

use std::rc::Rc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use sxo_types::Dialect;

use crate::{
    dialects::{HeldForm, dialect_from_str, dialect_to_str, map_err, parse_held},
    session::Session,
};
use athena::types::TermId;

/// Opaque expression handle backed by a shared host [`Session`].
///
/// Parse objects retain a dialect [`HeldForm`]. Evaluate / `d` / `simplify` stay on the
/// same session. Display renderers are never used as an execution serialization format.
#[derive(Debug)]
#[napi]
pub struct Expression {
    pub(crate) session: Rc<Session>,
    pub(crate) root: TermId,
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
        root: outcome.term,
        form: None,
        dialect,
        status: outcome.status,
        coverage: outcome.coverage,
        diagnostics: outcome.diagnostics,
    }
}

fn unevaluated(session: Rc<Session>, root: TermId, form: Option<HeldForm>, dialect: Dialect) -> Expression {
    Expression {
        session,
        root,
        form,
        dialect,
        status: "Unknown".into(),
        coverage: "Unknown".into(),
        diagnostics: Vec::new(),
    }
}

#[napi]
impl Expression {
    /// Parse `input` with an explicit dialect (`mathematica` | `matlab` | `simple-math`).
    #[napi(factory)]
    pub fn parse(input: String, dialect: Option<String>) -> Result<Self> {
        let d = dialect_from_str(dialect)?;
        let session = Rc::new(Session::new());
        let (root, form, resolved) = parse_held(&session, &input, d)?;
        Ok(unevaluated(session, root, Some(form), resolved))
    }

    /// Differentiate with respect to `var` on the same session.
    #[napi]
    pub fn d(&self, var: String) -> Result<Expression> {
        let root = self.session.differentiate_term(self.root, &var);
        Ok(unevaluated(Rc::clone(&self.session), root, None, self.dialect))
    }

    /// Simplify via the engine (`Simplify` head) on the same session.
    #[napi]
    pub fn simplify(&self) -> Result<Expression> {
        let root = self.session.simplify_term(self.root);
        Ok(unevaluated(Rc::clone(&self.session), root, None, self.dialect))
    }

    /// Evaluate from retained Form (parse objects) or arena term (result objects).
    #[napi]
    pub fn evaluate(&self) -> Result<Expression> {
        let outcome = match &self.form {
            Some(HeldForm::Matlab(form)) => self.session.evaluate_matlab_form(form),
            Some(HeldForm::Wolfram(form)) => self.session.evaluate_wolfram_form(form),
            None => self.session.evaluate_term_outcome(self.root),
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
        Ok(match self.dialect {
            Dialect::Matlab => self.session.render_as_matlab(self.root),
            _ => self.session.render_as_wolfram(self.root),
        })
    }

    /// Render as Mathematica / Wolfram text.
    #[napi(js_name = "toWolfram")]
    pub fn to_wolfram(&self) -> Result<String> {
        Ok(self.session.render_as_wolfram(self.root))
    }

    /// Render as MATLAB text.
    #[napi(js_name = "toMatlab")]
    pub fn to_matlab(&self) -> Result<String> {
        Ok(self.session.render_as_matlab(self.root))
    }

    /// Structural equality (Form round-trip compare).
    #[napi(js_name = "isEqual")]
    pub fn is_equal(&self, other: &Expression) -> Result<bool> {
        Ok(self.session.to_mathematica(self.root) == other.session.to_mathematica(other.root))
    }

    /// Render 1-D `Plot` / `plot` as SVG when the term matches a known form.
    #[napi(js_name = "plotSvg")]
    pub fn plot_svg(&self) -> Result<String> {
        match self.session.try_plot_svg(self.root, self.dialect) {
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
