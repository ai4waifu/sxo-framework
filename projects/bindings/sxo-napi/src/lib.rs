//! Node N-API bindings for SXO (`Session` + arena [`TermId`]).

#![deny(missing_docs)]

mod dialects;
mod handles;
mod jupyter;
pub mod session;

use std::rc::Rc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use session::Session;
use sxo_types::VERSION as CORE_VERSION;

use dialects::{dialect_from_str, map_err, parse_to_term};
use handles::Expression;

/// Return the SXO engine version string.
#[napi]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Top-level `d(expr, var, dialect?)`.
#[napi]
pub fn d(input: String, var: String, dialect: Option<String>) -> Result<Expression> {
    let d = dialect_from_str(dialect)?;
    let session = Rc::new(Session::new());
    let (term, resolved) = parse_to_term(&session, &input, d)?;
    let root = session.differentiate_term(term, &var);
    Ok(Expression {
        session,
        root,
        form: None,
        dialect: resolved,
        status: "Unknown".into(),
        coverage: "Unknown".into(),
        diagnostics: Vec::new(),
    })
}

/// Top-level `evaluate(expr, dialect?)` — parse + dialect `lower_request` path.
#[napi]
pub fn evaluate(input: String, dialect: Option<String>) -> Result<Expression> {
    let d = dialect_from_str(dialect)?;
    let session = Rc::new(Session::new());
    let outcome = session.evaluate_input(&input, d).map_err(map_err)?;
    Ok(Expression {
        session,
        root: outcome.term,
        form: None,
        dialect: d,
        status: outcome.status,
        coverage: outcome.coverage,
        diagnostics: outcome.diagnostics,
    })
}

/// Top-level `simplify(expr, dialect?)`.
#[napi]
pub fn simplify(input: String, dialect: Option<String>) -> Result<Expression> {
    let d = dialect_from_str(dialect)?;
    let session = Rc::new(Session::new());
    let outcome = session.evaluate_input(&input, d).map_err(map_err)?;
    let root = session.simplify_term(outcome.term);
    Ok(Expression {
        session,
        root,
        form: None,
        dialect: d,
        status: outcome.status,
        coverage: outcome.coverage,
        diagnostics: outcome.diagnostics,
    })
}

/// Top-level `expression(input, dialect?)` — parse only (no evaluate).
#[napi]
pub fn expression(input: String, dialect: Option<String>) -> Result<Expression> {
    Expression::parse(input, dialect)
}

/// Top-level `plotSvg(input, dialect?)` — 1-D plot → SVG string.
#[napi(js_name = "plotSvg")]
pub fn plot_svg(input: String, dialect: Option<String>) -> Result<String> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, &input, d)?;
    match session.try_plot_svg(term, resolved) {
        Some(Ok(svg)) => Ok(svg),
        Some(Err(e)) => Err(map_err(e)),
        None => Err(Error::from_reason("not a supported 1-D plot form")),
    }
}

/// Run a Jupyter kernel until shutdown (blocks the calling thread).
#[napi]
pub fn run_jupyter_kernel(connection_file: String) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::from_reason(format!("tokio runtime: {e}")))?;
    rt.block_on(jupyter::run(&connection_file)).map_err(Error::from_reason)
}
