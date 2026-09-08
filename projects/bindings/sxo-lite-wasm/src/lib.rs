//! WASM bindings for SXO (`Session` + arena [`TermId`]).

#![deny(missing_docs)]

mod dialects;
mod handles;
mod options;
mod session;

use std::rc::Rc;

use session::Session;
use sxo_types::VERSION as CORE_VERSION;
use wasm_bindgen::prelude::*;

use dialects::{dialect_from_str, map_err, parse_to_term};
use handles::{Expression, from_outcome};
use options::parse_strategy;

/// Return the SXO engine version string.
#[wasm_bindgen]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Top-level `evaluate` — parse + dialect `lower_request` path.
///
/// `strategy`: `"none"` (default) or `"simplify"`.
#[wasm_bindgen]
pub fn evaluate(input: &str, dialect: Option<String>, strategy: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let strategy = parse_strategy(strategy.as_deref())?;
    let session = Rc::new(Session::new());
    let outcome = session.evaluate_input(input, d).map_err(map_err)?;
    from_outcome(session, d, outcome, strategy)
}

/// Top-level `d`.
#[wasm_bindgen]
pub fn d(input: &str, var: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    let d = dialect_from_str(dialect)?;
    let session = Rc::new(Session::new());
    let (term, resolved) = parse_to_term(&session, input, d)?;
    let outcome = session.differentiate_outcome(term, var).map_err(map_err)?;
    Ok(Expression {
        session,
        root: None,
        result_id: Some(outcome.result_id),
        form: None,
        dialect: resolved,
    })
}

/// Top-level `simplify`.
#[wasm_bindgen]
pub fn simplify(input: &str, dialect: Option<String>) -> Result<Expression, JsValue> {
    evaluate(input, dialect, Some("simplify".into()))
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
