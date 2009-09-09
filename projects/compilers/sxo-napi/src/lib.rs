//! Node N-API bindings for SXO (`SxoEngine` + engine [`Term`]).
//!
//! Mathematica callers go through Wolfram text ↔ `WExpr` ↔ `Term` inside the engine.
//! This crate does **not** expose `WExpr` to JS.

#![deny(missing_docs)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use sxo_engine::{Dialect, SxoEngine, SxoError, Term, VERSION as CORE_VERSION};

fn map_err(err: SxoError) -> Error {
    Error::from_reason(err.message)
}

fn dialect_from_str(s: Option<String>) -> Result<Dialect> {
    match s.as_deref() {
        None | Some("auto") => Ok(Dialect::Auto),
        Some("simple-math") | Some("sm") => Ok(Dialect::SimpleMath),
        Some("mathematica") => Ok(Dialect::Mathematica),
        Some("matlab") => Ok(Dialect::Matlab),
        Some(other) => Err(Error::from_reason(format!("unknown dialect: {other}"))),
    }
}

fn dialect_to_str(d: Dialect) -> &'static str {
    match d {
        Dialect::Auto => "auto",
        Dialect::SimpleMath => "simple-math",
        Dialect::Mathematica => "mathematica",
        Dialect::Matlab => "matlab",
    }
}

fn parse_to_term(eng: &SxoEngine, input: &str, dialect: Dialect) -> Result<(Term, Dialect)> {
    let resolved = match eng.resolve_dialect(input, dialect) {
        Dialect::Auto => Dialect::Mathematica,
        other => other,
    };
    let term = match resolved {
        Dialect::Mathematica => {
            let w = eng.parse_mathematica(input).map_err(map_err)?;
            eng.from_mathematica(&w)
        }
        Dialect::Matlab => eng.parse_matlab(input).map_err(map_err)?,
        Dialect::SimpleMath | Dialect::Auto => {
            return Err(Error::from_reason("simple-math dialect is off the current delivery route"));
        }
    };
    Ok((term, resolved))
}

/// Return the SXO engine version string.
#[napi]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Opaque expression handle backed by engine [`Term`].
#[derive(Debug)]
#[napi]
pub struct Expression {
    inner: Term,
    dialect: Dialect,
}

#[napi]
impl Expression {
    /// Parse `input` with optional dialect (`auto` | `mathematica` | `matlab`).
    #[napi(factory)]
    pub fn parse(input: String, dialect: Option<String>) -> Result<Self> {
        let d = dialect_from_str(dialect)?;
        let eng = SxoEngine::new();
        let (term, resolved) = parse_to_term(&eng, &input, d)?;
        Ok(Self { inner: term, dialect: resolved })
    }

    /// Differentiate with respect to `var`.
    #[napi]
    pub fn d(&self, var: String) -> Result<Expression> {
        let eng = SxoEngine::new();
        let out = eng.differentiate_term(&self.inner, &var);
        Ok(Self { inner: out, dialect: self.dialect })
    }

    /// Simplify via the engine (`Simplify` head).
    #[napi]
    pub fn simplify(&self) -> Result<Expression> {
        let eng = SxoEngine::new();
        Ok(Self { inner: eng.simplify_term(&self.inner), dialect: self.dialect })
    }

    /// Evaluate (canonical rewrite) this expression.
    #[napi]
    pub fn evaluate(&self) -> Result<Expression> {
        let eng = SxoEngine::new();
        Ok(Self { inner: eng.evaluate(&self.inner), dialect: self.dialect })
    }

    /// Render as string in the expression's dialect.
    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> Result<String> {
        let eng = SxoEngine::new();
        Ok(match self.dialect {
            Dialect::Matlab => eng.render_as_matlab(&self.inner),
            _ => eng.render_as_wolfram(&self.inner),
        })
    }

    /// Render as Mathematica / Wolfram text.
    #[napi(js_name = "toWolfram")]
    pub fn to_wolfram(&self) -> Result<String> {
        let eng = SxoEngine::new();
        Ok(eng.render_as_wolfram(&self.inner))
    }

    /// Render as MATLAB text.
    #[napi(js_name = "toMatlab")]
    pub fn to_matlab(&self) -> Result<String> {
        let eng = SxoEngine::new();
        Ok(eng.render_as_matlab(&self.inner))
    }

    /// Structural equality.
    #[napi(js_name = "isEqual")]
    pub fn is_equal(&self, other: &Expression) -> Result<bool> {
        Ok(self.inner == other.inner)
    }

    /// Dialect tag used for default rendering.
    #[napi(getter)]
    pub fn dialect(&self) -> String {
        dialect_to_str(self.dialect).to_string()
    }
}

/// Top-level `d(expr, var, dialect?)`.
#[napi]
pub fn d(input: String, var: String, dialect: Option<String>) -> Result<Expression> {
    let d = dialect_from_str(dialect)?;
    let eng = SxoEngine::new();
    let (term, resolved) = parse_to_term(&eng, &input, d)?;
    Ok(Expression { inner: eng.differentiate_term(&term, &var), dialect: resolved })
}

/// Top-level `simplify(expr, dialect?)`.
#[napi]
pub fn simplify(input: String, dialect: Option<String>) -> Result<Expression> {
    let d = dialect_from_str(dialect)?;
    let eng = SxoEngine::new();
    let (term, resolved) = parse_to_term(&eng, &input, d)?;
    Ok(Expression { inner: eng.simplify_term(&eng.evaluate(&term)), dialect: resolved })
}

/// Top-level `expression(input, dialect?)` — parse only (no evaluate).
#[napi]
pub fn expression(input: String, dialect: Option<String>) -> Result<Expression> {
    Expression::parse(input, dialect)
}
