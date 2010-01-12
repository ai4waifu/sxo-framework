//! Opaque WASM expression handles.

use sxo_types::Dialect;
use wasm_bindgen::prelude::*;

use crate::dialects::{dialect_from_str, map_err, parse_to_term};
use crate::session::Session;
use athena::types::TermId;

/// Opaque expression handle backed by a host [`Session`] arena [`TermId`].
#[derive(Debug)]
#[wasm_bindgen]
pub struct Expression {
    pub(crate) session: Session,
    pub(crate) root: TermId,
    pub(crate) dialect: Dialect,
}

fn fork_expression(session: &Session, root: TermId, dialect: Dialect) -> Result<Expression, JsValue> {
    let fresh = Session::new();
    let root = match dialect {
        Dialect::Matlab => {
            let text = session.render_as_matlab(root);
            fresh.parse_matlab(&text).map_err(map_err)?
        }
        Dialect::Mathematica | Dialect::SimpleMath => {
            let w = session.to_mathematica(root);
            fresh.lower_mathematica(&w)
        }
    };
    Ok(Expression { session: fresh, root, dialect })
}

#[wasm_bindgen]
impl Expression {
    /// Parse `input` with an explicit dialect (`mathematica` | `matlab` | `simple-math`).
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
        let d = dialect_from_str(dialect)?;
        let session = Session::new();
        let (root, resolved) = parse_to_term(&session, input, d)?;
        Ok(Self { session, root, dialect: resolved })
    }

    /// Differentiate with respect to `var`.
    pub fn d(&self, var: &str) -> Result<Expression, JsValue> {
        let mut out = fork_expression(&self.session, self.root, self.dialect)?;
        out.root = out.session.differentiate_term(out.root, var);
        Ok(out)
    }

    /// Simplify via `Session`.
    pub fn simplify(&self) -> Result<Expression, JsValue> {
        let mut out = fork_expression(&self.session, self.root, self.dialect)?;
        out.root = out.session.simplify_term(out.root);
        Ok(out)
    }

    /// Evaluate via dialect `lower_request` (Session / Control / Domain Goals).
    pub fn evaluate(&self) -> Result<Expression, JsValue> {
        let mut out = fork_expression(&self.session, self.root, self.dialect)?;
        out.root = out.session.evaluate_form(out.root, out.dialect).map_err(map_err)?;
        Ok(out)
    }

    /// Render as string in the expression's dialect.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        match self.dialect {
            Dialect::Matlab => self.session.render_as_matlab(self.root),
            _ => self.session.render_as_wolfram(self.root),
        }
    }

    /// Render as Mathematica / Wolfram text.
    #[wasm_bindgen(js_name = toWolfram)]
    pub fn to_wolfram(&self) -> String {
        self.session.render_as_wolfram(self.root)
    }

    /// Render as MATLAB text.
    #[wasm_bindgen(js_name = toMatlab)]
    pub fn to_matlab(&self) -> String {
        self.session.render_as_matlab(self.root)
    }

    /// Structural equality (Form round-trip compare).
    #[wasm_bindgen(js_name = isEqual)]
    pub fn is_equal(&self, other: &Expression) -> bool {
        self.session.to_mathematica(self.root) == other.session.to_mathematica(other.root)
    }

    /// Render 1-D plot as SVG when the term matches a known form.
    #[wasm_bindgen(js_name = plotSvg)]
    pub fn plot_svg(&self) -> Result<String, JsValue> {
        match self.session.try_plot_svg(self.root, self.dialect) {
            Some(Ok(svg)) => Ok(svg),
            Some(Err(e)) => Err(map_err(e)),
            None => Err(JsValue::from_str("not a supported 1-D plot form")),
        }
    }
}
