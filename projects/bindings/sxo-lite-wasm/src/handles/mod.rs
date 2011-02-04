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
/// evaluate / `simplify` / plot needs them (`d` uses Form → `lower_request` when Form is present).
/// Display uses Form renderers when present.
/// Evaluate results retain a Session-local [`ResultId`] and project [`TermId`] on demand.
#[derive(Debug)]
#[wasm_bindgen]
pub struct Expression {
    pub(crate) session: Rc<Session>,
    /// Present only when a bare term was materialized without a Session result (should be rare).
    pub(crate) root: Option<TermId>,
    /// Present after evaluate / `d` / `simplify`.
    pub(crate) result_id: Option<ResultId>,
    /// Present on parse objects. Cleared on evaluate / `d` / `simplify` results.
    pub(crate) form: Option<HeldForm>,
    pub(crate) dialect: Dialect,
    /// Athena [ComputationStatus] name from the owning result (or Unknown if not evaluated).
    pub(crate) status: String,
    /// Coverage name from the owning result (or Unknown if not evaluated).
    pub(crate) coverage: String,
}

/// Build an [`Expression`] from an evaluate outcome, optionally applying Simplify in-process.
///
/// `simplify` replaces the outcome with the Simplify request's final [`ResultId`],
/// linked via `derived_from` to the evaluate result.
pub(crate) fn from_outcome(
    session: Rc<Session>,
    dialect: Dialect,
    outcome: sxo_types::EvalOutcome,
    strategy: EvalStrategy,
) -> Result<Expression, JsValue> {
    let final_outcome = match strategy {
        EvalStrategy::None => outcome,
        EvalStrategy::Simplify => session.simplify_result(outcome.result_id).map_err(map_err)?,
    };
    Ok(Expression {
        session,
        root: None,
        result_id: Some(final_outcome.result_id),
        form: None,
        dialect,
        status: final_outcome.status,
        coverage: final_outcome.coverage,
    })
}

pub(crate) fn from_eval_outcome(session: Rc<Session>, dialect: Dialect, outcome: sxo_types::EvalOutcome) -> Expression {
    Expression {
        session,
        root: None,
        result_id: Some(outcome.result_id),
        form: None,
        dialect,
        status: outcome.status,
        coverage: outcome.coverage,
    }
}

impl Expression {
    /// Prefer projecting `result_id` when present.
    fn materialize_root(&self) -> Result<TermId, JsValue> {
        if let Some(result_id) = self.result_id {
            return self.session.try_project_symbolic(result_id).map_err(map_err);
        }
        if let Some(root) = self.root {
            return Ok(root);
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
            status: "Unknown".into(),
            coverage: "Unknown".into(),
        })
    }

    /// Differentiate with respect to `var` on the same session.
    ///
    /// Parse objects wrap retained Form as dialect `D` / `diff` and run `lower_request`.
    /// Result objects go through `differentiate_result`: one outcome, `derived_from` recorded there.
    pub fn d(&self, var: &str) -> Result<Expression, JsValue> {
        let parent = self.result_id;
        let outcome = match &self.form {
            Some(HeldForm::Wolfram(form)) => self.session.differentiate_wolfram_form(form, var),
            Some(HeldForm::Matlab(form)) => self.session.differentiate_matlab_form(form, var),
            None => {
                if let Some(parent) = self.result_id {
                    self.session.differentiate_result(parent, var)
                }
                else {
                    let term = self.materialize_root()?;
                    self.session.differentiate_outcome(term, var)
                }
            }
        }
        .map_err(map_err)?;
        let linked_inside = self.form.is_none() && self.result_id.is_some();
        if let Some(parent) = parent {
            if !linked_inside {
                let _ = self.session.link_derived_from(outcome.result_id, parent);
            }
        }
        Ok(from_eval_outcome(Rc::clone(&self.session), self.dialect, outcome))
    }

    /// Simplify via `Session` on the same session.
    ///
    /// Parse objects wrap retained Form as dialect `Simplify` and run `lower_request`.
    /// Result objects go through `simplify_result`: one outcome, `derived_from` recorded there.
    pub fn simplify(&self) -> Result<Expression, JsValue> {
        let parent = self.result_id;
        let outcome = match &self.form {
            Some(HeldForm::Wolfram(form)) => self.session.simplify_wolfram_form(form),
            Some(HeldForm::Matlab(form)) => self.session.simplify_matlab_form(form),
            None => {
                if let Some(parent) = self.result_id {
                    self.session.simplify_result(parent)
                }
                else {
                    let term = self.materialize_root()?;
                    self.session.simplify_outcome(term)
                }
            }
        }
        .map_err(map_err)?;
        let linked_inside = self.form.is_none() && self.result_id.is_some();
        if let Some(parent) = parent {
            if !linked_inside {
                let _ = self.session.link_derived_from(outcome.result_id, parent);
            }
        }
        Ok(from_eval_outcome(Rc::clone(&self.session), self.dialect, outcome))
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
        from_outcome(Rc::clone(&self.session), self.dialect, outcome, strategy)
    }

    /// Athena computation status name (`Exact`, `Candidate`, `Unknown`, …).
    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String {
        self.status.clone()
    }

    /// Coverage name (`Full`, `Partial`, `Unknown`, `Unsupported`).
    #[wasm_bindgen(getter)]
    pub fn coverage(&self) -> String {
        self.coverage.clone()
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

    /// Condition summaries from the last evaluate (empty if none / not evaluated).
    #[wasm_bindgen(getter)]
    pub fn conditions(&self) -> Vec<String> {
        match self.result_id {
            Some(id) => self.session.project_conditions(id),
            None => Vec::new(),
        }
    }

    /// Provider stamp from the last evaluate (`Name@vN`, or `undefined` if none / not evaluated).
    #[wasm_bindgen(getter)]
    pub fn provider(&self) -> Option<String> {
        self.result_id.and_then(|id| self.session.project_provider(id))
    }

    /// Evidence summaries from the last evaluate (empty if none / not evaluated).
    #[wasm_bindgen(getter)]
    pub fn evidence(&self) -> Vec<String> {
        match self.result_id {
            Some(id) => self.session.project_evidence(id),
            None => Vec::new(),
        }
    }

    /// Parent evaluate result id when this handle is a Simplify transform, else `undefined`.
    #[wasm_bindgen(getter, js_name = derivedFrom)]
    pub fn derived_from_js(&self) -> Option<u32> {
        self.result_id.and_then(|id| self.session.derived_from(id).map(|parent| parent.0))
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
