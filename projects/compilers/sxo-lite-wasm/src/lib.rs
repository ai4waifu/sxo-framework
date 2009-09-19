//! WASM bindings for SXO (`Session` + engine [`Term`]).

#![deny(missing_docs)]

mod session;

use session::Session;
use sxo_types::{Dialect, SxoError, VERSION as CORE_VERSION};
use wasm_bindgen::prelude::*;

pub use athena::Term;

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

fn parse_to_term(session: &Session, input: &str, dialect: Dialect) -> Result<(Term, Dialect), JsValue> {
    let resolved = match session.resolve_dialect(input, dialect) {
        Dialect::Auto => Dialect::Mathematica,
        other => other,
    };
    let term = match resolved {
        Dialect::Mathematica => {
            let w = session.parse_mathematica(input).map_err(map_err)?;
            session.from_mathematica(&w)
        }
        Dialect::Matlab => session.parse_matlab(input).map_err(map_err)?,
        Dialect::SimpleMath | Dialect::Auto => {
            return Err(JsValue::from_str("simple-math dialect is off the current delivery route"));
        }
    };
    Ok((term, resolved))
}

/// Return the SXO engine version string.
#[wasm_bindgen]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Opaque expression handle backed by engine [`Term`].
#[derive(Debug)]
#[wasm_bindgen]
pub struct Expression {
    inner: Term,
    dialect: Dialect,
}

#[wasm_bindgen]
impl Expression {
    /// Parse `input` with optional dialect.
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
        let d = dialect_from_str(dialect)?;
        let session = Session::new();
        let (term, resolved) = parse_to_term(&session, input, d)?;
        Ok(Self { inner: term, dialect: resolved })
    }

    /// Differentiate with respect to `var`.
    pub fn d(&self, var: &str) -> Expression {
        let session = Session::new();
        Expression { inner: session.differentiate_term(&self.inner, var), dialect: self.dialect }
    }

    /// Simplify via `Session`.
    pub fn simplify(&self) -> Expression {
        let session = Session::new();
        Expression { inner: session.simplify_term(&session.evaluate(&self.inner)), dialect: self.dialect }
    }

    /// Evaluate (canonical rewrite) this expression.
    pub fn evaluate(&self) -> Expression {
        let session = Session::new();
        Expression { inner: session.evaluate(&self.inner), dialect: self.dialect }
    }

    /// Render as string in the expression's dialect.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        let session = Session::new();
        match self.dialect {
            Dialect::Matlab => session.render_as_matlab(&self.inner),
            _ => session.render_as_wolfram(&self.inner),
        }
    }

    /// Render as Mathematica / Wolfram text.
    #[wasm_bindgen(js_name = toWolfram)]
    pub fn to_wolfram(&self) -> String {
        let session = Session::new();
        session.render_as_wolfram(&self.inner)
    }

    /// Render as MATLAB text.
    #[wasm_bindgen(js_name = toMatlab)]
    pub fn to_matlab(&self) -> String {
        let session = Session::new();
        session.render_as_matlab(&self.inner)
    }

    /// Structural equality.
    #[wasm_bindgen(js_name = isEqual)]
    pub fn is_equal(&self, other: &Expression) -> bool {
        self.inner == other.inner
    }
}

/// Top-level `d`.
#[wasm_bindgen]
pub fn d(input: &str, var: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, input, d)?;
    Ok(Expression { inner: session.differentiate_term(&term, var), dialect: resolved })
}

/// Top-level `simplify`.
#[wasm_bindgen]
pub fn simplify(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, input, d)?;
    Ok(Expression { inner: session.simplify_term(&session.evaluate(&term)), dialect: resolved })
}

/// Top-level `expression` — parse only (no evaluate).
#[wasm_bindgen]
pub fn expression(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    Expression::new(input, dialect)
}
