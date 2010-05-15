//! Opaque WASM expression handles.

use std::rc::Rc;

use sxo_types::Dialect;
use wasm_bindgen::prelude::*;

use crate::{
    dialects::{HeldForm, dialect_from_str, map_err, parse_held, render_held},
    session::Session,
};
use athena::types::TermId;

/// Opaque expression handle backed by a shared host [`Session`].
///
/// Parse objects retain a dialect Form and do not materialize arena terms until
/// evaluate / `d` / `simplify` / plot needs them. Display uses Form renderers when present.
#[derive(Debug)]
#[wasm_bindgen]
pub struct Expression {
    pub(crate) session: Rc<Session>,
    /// Present after evaluate / `d` / `simplify` (or string-entry helpers that materialize).
    pub(crate) root: Option<TermId>,
    /// Present on parse objects. Cleared on evaluate / `d` / `simplify` results.
    pub(crate) form: Option<HeldForm>,
    pub(crate) dialect: Dialect,
}

impl Expression {
    fn materialize_root(&self) -> Result<TermId, JsValue> {
        if let Some(root) = self.root {
            return Ok(root);
        }
        match &self.form {
            Some(HeldForm::Matlab(form)) => Ok(self.session.lower_matlab(form)),
            Some(HeldForm::Wolfram(form)) => Ok(self.session.lower_mathematica(form)),
            None => Err(JsValue::from_str("expression has neither Form nor Term root")),
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
            form: None,
            dialect: self.dialect,
        })
    }

    /// Evaluate from retained Form (parse objects) or arena term (result objects).
    pub fn evaluate(&self) -> Result<Expression, JsValue> {
        let root = match &self.form {
            Some(HeldForm::Matlab(form)) => self.session.evaluate_matlab_form(form),
            Some(HeldForm::Wolfram(form)) => self.session.evaluate_wolfram_form(form),
            None => {
                let root = self.root.ok_or_else(|| JsValue::from_str("expression has neither Form nor Term root"))?;
                self.session.evaluate_term(root)
            }
        }
        .map_err(map_err)?;
        Ok(Expression {
            session: Rc::clone(&self.session),
            root: Some(root),
            form: None,
            dialect: self.dialect,
        })
    }

    /// Render as string in the expression's dialect.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> Result<String, JsValue> {
        if let Some(form) = &self.form {
            return Ok(render_held(form));
        }
        let root = self.root.ok_or_else(|| JsValue::from_str("expression has neither Form nor Term root"))?;
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
