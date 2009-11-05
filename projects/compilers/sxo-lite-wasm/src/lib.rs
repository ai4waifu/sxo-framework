//! WASM bindings for SXO (`Session` + arena [`TermId`]).

#![deny(missing_docs)]

mod session;

use athena::types::TermId;
use session::Session;
use sxo_types::{Dialect, SxoError, VERSION as CORE_VERSION};
use wasm_bindgen::prelude::*;

fn dialect_from_str(s: Option<String>) -> Result<Dialect, JsValue> {
    match s.as_deref() {
        None | Some("auto") => Ok(Dialect::Auto),
        Some("simple-math") | Some("sm") => Ok(Dialect::SimpleMath),
        Some("mathematica") => Ok(Dialect::Mathematica),
        Some("matlab") => Ok(Dialect::Matlab),
        Some(other) => Err(JsValue::from_str(&format!("unknown dialect: {other}"))),
    }
}

fn map_err(err: SxoError) -> JsValue {
    JsValue::from_str(&err.message)
}

fn parse_to_term(session: &Session, input: &str, dialect: Dialect) -> Result<(TermId, Dialect), JsValue> {
    let resolved = match session.resolve_dialect(input, dialect) {
        Dialect::Auto => Dialect::Mathematica,
        other => other,
    };
    let term = match resolved {
        Dialect::Mathematica => {
            let w = session.parse_mathematica(input).map_err(map_err)?;
            session.lower_mathematica(&w)
        }
        Dialect::Matlab => session.parse_matlab(input).map_err(map_err)?,
        Dialect::SimpleMath | Dialect::Auto => {
            return Err(JsValue::from_str("simple-math dialect is off the current delivery route"));
        }
    };
    Ok((term, resolved))
}

fn fork_expression(session: &Session, root: TermId, dialect: Dialect) -> Expression {
    let w = session.to_mathematica(root);
    let fresh = Session::new();
    let root = fresh.lower_mathematica(&w);
    Expression { session: fresh, root, dialect }
}

/// Return the SXO engine version string.
#[wasm_bindgen]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Opaque expression handle backed by a host [`Session`] arena [`TermId`].
#[derive(Debug)]
#[wasm_bindgen]
pub struct Expression {
    session: Session,
    root: TermId,
    dialect: Dialect,
}

#[wasm_bindgen]
impl Expression {
    /// Parse `input` with optional dialect.
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
        let d = dialect_from_str(dialect)?;
        let session = Session::new();
        let (root, resolved) = parse_to_term(&session, input, d)?;
        Ok(Self { session, root, dialect: resolved })
    }

    /// Differentiate with respect to `var`.
    pub fn d(&self, var: &str) -> Expression {
        let mut out = fork_expression(&self.session, self.root, self.dialect);
        out.root = out.session.differentiate_term(out.root, var);
        out
    }

    /// Simplify via `Session`.
    pub fn simplify(&self) -> Expression {
        let mut out = fork_expression(&self.session, self.root, self.dialect);
        let evaluated = out.session.evaluate(out.root);
        out.root = out.session.simplify_term(evaluated);
        out
    }

    /// Evaluate (canonical rewrite) this expression.
    pub fn evaluate(&self) -> Expression {
        let mut out = fork_expression(&self.session, self.root, self.dialect);
        out.root = out.session.evaluate(out.root);
        out
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

/// Top-level `d`.
#[wasm_bindgen]
pub fn d(input: &str, var: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, input, d)?;
    let root = session.differentiate_term(term, var);
    Ok(Expression { session, root, dialect: resolved })
}

/// Top-level `simplify`.
#[wasm_bindgen]
pub fn simplify(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, input, d)?;
    let evaluated = session.evaluate(term);
    let root = session.simplify_term(evaluated);
    Ok(Expression { session, root, dialect: resolved })
}

/// Top-level `expression` — parse only (no evaluate).
#[wasm_bindgen]
pub fn expression(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    Expression::new(input, dialect)
}

/// Top-level `plotSvg` — 1-D plot → SVG string.
#[wasm_bindgen(js_name = plotSvg)]
pub fn plot_svg(input: &str, dialect: Option<String>) -> Result<String, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, input, d)?;
    match session.try_plot_svg(term, resolved) {
        Some(Ok(svg)) => Ok(svg),
        Some(Err(e)) => Err(map_err(e)),
        None => Err(JsValue::from_str("not a supported 1-D plot form")),
    }
}
