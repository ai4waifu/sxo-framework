//! Opaque WASM expression handles.

use std::rc::Rc;

use sxo_types::Dialect;
use wasm_bindgen::prelude::*;

use crate::{
    dialects::{HeldForm, dialect_from_str, map_err, parse_held, render_held},
    options::{EvalStrategy, parse_strategy},
    session::Session,
};
use athena::types::{ResultId, TermId};

/// Opaque expression handle backed by a shared host [`Session`].
///
/// Parse objects retain a dialect Form and do not materialize arena terms until
/// evaluate / `d` / `simplify` / plot needs them. Display uses Form renderers when present.
/// Evaluate results retain a Session-local [`ResultId`] and project [`TermId`] on demand.
#[derive(Debug)]
#[wasm_bindgen]
pub struct Expression {
    pub(crate) session: Rc<Session>,
    /// Present after `d` / `simplify` (or string-entry helpers that materialize a bare term).
    pub(crate) root: Option<TermId>,
    /// Present after evaluate. Prefer over `root` when projecting symbolic results.
    pub(crate) result_id: Option<ResultId>,
    /// Present on parse objects. Cleared on evaluate / `d` / `simplify` results.
    pub(crate) form: Option<HeldForm>,
    pub(crate) dialect: Dialect,
}

/// Build an [`Expression`] from an evaluate outcome, optionally applying Simplify in-process.
pub(crate) fn from_outcome(
    session: Rc<Session>,
    dialect: Dialect,
    outcome: sxo_types::EvalOutcome,
    strategy: EvalStrategy,
) -> Expression {
    match strategy {
        EvalStrategy::None => Expression {
            session,
            root: None,
            result_id: Some(outcome.result_id),
            form: None,
            dialect,
        },
        EvalStrategy::Simplify => {
            let root = session.simplify_term(session.project_result(outcome.result_id));
            Expression {
                session,
                root: Some(root),
                // Keep ResultId so diagnostics can stay lazy after in-process simplify.
                result_id: Some(outcome.result_id),
                form: None,
                dialect,
            }
        }
    }
}

impl Expression {
    /// Prefer an explicit `root` (e.g. post-simplify) over projecting `result_id`.
    fn materialize_root(&self) -> Result<TermId, JsValue> {
        if let Some(root) = self.root {
            return Ok(root);
        }
        if let Some(result_id) = self.result_id {
            return Ok(self.session.project_result(result_id));
        }
        match &self.form {
            Some(HeldForm::Matlab(form)) => Ok(self.session.lower_matlab(form)),
            Some(HeldForm::Wolfram(form)) => Ok(self.session.lower_mathematica(form)),
            None => Err(JsValue::from_str("expression has neither Form, ResultId, nor Term root")),
        }
    }
}

#[wasm_bindgen]
impl Expression {
    /// Parse `input` with an explicit dialect (`mathematica` | `matlab` | `simple-math`).
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
        let d = dialect_from_str(dialect)?;
        let session = Rc::new(Session::new());
        let (form, resolved) = parse_held(&session, input, d)?;
        Ok(Self {
            session,
            root: None,
            result_id: None,
            form: Some(form),
            dialect: resolved,
        })
    }

    /// Differentiate with respect to `var` on the same session.
    pub fn d(&self, var: &str) -> Result<Expression, JsValue> {
        let term = self.materialize_root()?;
        let root = self.session.differentiate_term(term, var);
        Ok(Expression {
            session: Rc::clone(&self.session),
            root: Some(root),
            result_id: None,
            form: None,
            dialect: self.dialect,
        })
    }

    /// Simplify via `Session` on the same session.
    pub fn simplify(&self) -> Result<Expression, JsValue> {
        let term = self.materialize_root()?;
        let root = self.session.simplify_term(term);
        Ok(Expression {
            session: Rc::clone(&self.session),
            root: Some(root),
            result_id: None,
            form: None,
            dialect: self.dialect,
        })
    }

    /// Evaluate from retained Form (parse objects) or arena term (result objects).
    ///
    /// `strategy`: `"none"` (default) or `"simplify"` — applied in this crossing.
    pub fn evaluate(&self, strategy: Option<String>) -> Result<Expression, JsValue> {
        let strategy = parse_strategy(strategy.as_deref())?;
        let outcome = match &self.form {
            Some(HeldForm::Matlab(form)) => self.session.evaluate_matlab_form(form),
            Some(HeldForm::Wolfram(form)) => self.session.evaluate_wolfram_form(form),
            None => {
                let root = self.materialize_root()?;
                self.session.evaluate_term_outcome(root)
            }
        }
        .map_err(map_err)?;
        Ok(from_outcome(Rc::clone(&self.session), self.dialect, outcome, strategy))
    }

    /// Diagnostic summaries from the last evaluate (empty if none / not evaluated).
    ///
    /// Projected from the Session [`ResultId`] on demand — not copied at evaluate time.
    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> Vec<String> {
        match self.result_id {
            Some(id) => self.session.project_diagnostics(id),
            None => Vec::new(),
        }
    }

    /// Render as string in the expression's dialect.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> Result<String, JsValue> {
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
    #[wasm_bindgen(js_name = toWolfram)]
    pub fn to_wolfram(&self) -> Result<String, JsValue> {
        if let Some(HeldForm::Wolfram(w)) = &self.form {
            return Ok(sxo_dialect_mathematica::render(w));
        }
        let root = self.materialize_root()?;
        Ok(self.session.render_as_wolfram(root))
    }

    /// Render as MATLAB text.
    #[wasm_bindgen(js_name = toMatlab)]
    pub fn to_matlab(&self) -> Result<String, JsValue> {
        if let Some(HeldForm::Matlab(f)) = &self.form {
            return Ok(sxo_dialect_matlab::render_matlab_form(f));
        }
        let root = self.materialize_root()?;
        Ok(self.session.render_as_matlab(root))
    }

    /// Structural equality (Form compare when both held; else Term via Mathematica projection).
    #[wasm_bindgen(js_name = isEqual)]
    pub fn is_equal(&self, other: &Expression) -> Result<bool, JsValue> {
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

    /// Render 1-D plot as SVG when the term matches a known form.
    #[wasm_bindgen(js_name = plotSvg)]
    pub fn plot_svg(&self) -> Result<String, JsValue> {
        let root = self.materialize_root()?;
        match self.session.try_plot_svg(root, self.dialect) {
            Some(Ok(svg)) => Ok(svg),
            Some(Err(e)) => Err(map_err(e)),
            None => Err(JsValue::from_str("not a supported 1-D plot form")),
        }
    }
}
