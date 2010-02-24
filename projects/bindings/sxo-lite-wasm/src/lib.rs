//! WASM bindings for SXO (`Session` + arena [`TermId`]).

#![deny(missing_docs)]

mod dialects;
mod handles;
mod session;

use handles::Expression;
use session::Session;
use sxo_types::VERSION as CORE_VERSION;
use wasm_bindgen::prelude::*;

use dialects::{dialect_from_str, map_err, parse_to_term};

/// Return the SXO engine version string.
#[wasm_bindgen]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Top-level `evaluate` — parse + dialect `lower_request` path.
#[wasm_bindgen]
pub fn evaluate(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let root = session.evaluate_input(input, d).map_err(map_err)?;
    Ok(Expression { session, root, dialect: d })
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
    let evaluated = session.evaluate_input(input, d).map_err(map_err)?;
    let root = session.simplify_term(evaluated);
    Ok(Expression { session, root, dialect: d })
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
