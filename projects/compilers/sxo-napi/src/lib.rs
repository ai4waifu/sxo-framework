//! Node N-API bindings for SXO (`Session` + arena [`TermId`]).

#![deny(missing_docs)]

mod jupyter;
pub mod session;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use session::Session;
use sxo_types::{Dialect, SxoError, VERSION as CORE_VERSION};

use athena::types::TermId;

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

fn parse_to_term(session: &Session, input: &str, dialect: Dialect) -> Result<(TermId, Dialect)> {
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
            return Err(Error::from_reason("simple-math dialect is off the current delivery route"));
        }
    };
    Ok((term, resolved))
}

/// Fork `root` into a fresh host [`Session`] via Mathematica Form round-trip.
fn fork_expression(session: &Session, root: TermId, dialect: Dialect) -> Expression {
    let w = session.to_mathematica(root);
    let fresh = Session::new();
    let root = fresh.lower_mathematica(&w);
    Expression { session: fresh, root, dialect }
}

/// Return the SXO engine version string.
#[napi]
pub fn version() -> String {
    CORE_VERSION.to_string()
}

/// Opaque expression handle backed by a host [`Session`] arena [`TermId`].
#[derive(Debug)]
#[napi]
pub struct Expression {
    session: Session,
    root: TermId,
    dialect: Dialect,
}

#[napi]
impl Expression {
    /// Parse `input` with optional dialect (`auto` | `mathematica` | `matlab`).
    #[napi(factory)]
    pub fn parse(input: String, dialect: Option<String>) -> Result<Self> {
        let d = dialect_from_str(dialect)?;
        let session = Session::new();
        let (root, resolved) = parse_to_term(&session, &input, d)?;
        Ok(Self { session, root, dialect: resolved })
    }

    /// Differentiate with respect to `var`.
    #[napi]
    pub fn d(&self, var: String) -> Result<Expression> {
        let mut out = fork_expression(&self.session, self.root, self.dialect);
        out.root = out.session.differentiate_term(out.root, &var);
        Ok(out)
    }

    /// Simplify via the engine (`Simplify` head).
    #[napi]
    pub fn simplify(&self) -> Result<Expression> {
        let mut out = fork_expression(&self.session, self.root, self.dialect);
        out.root = out.session.simplify_term(out.root);
        Ok(out)
    }

    /// Evaluate (canonical rewrite) this expression.
    #[napi]
    pub fn evaluate(&self) -> Result<Expression> {
        let mut out = fork_expression(&self.session, self.root, self.dialect);
        out.root = out.session.evaluate(out.root);
        Ok(out)
    }

    /// Render as string in the expression's dialect.
    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> Result<String> {
        Ok(match self.dialect {
            Dialect::Matlab => self.session.render_as_matlab(self.root),
            _ => self.session.render_as_wolfram(self.root),
        })
    }

    /// Render as Mathematica / Wolfram text.
    #[napi(js_name = "toWolfram")]
    pub fn to_wolfram(&self) -> Result<String> {
        Ok(self.session.render_as_wolfram(self.root))
    }

    /// Render as MATLAB text.
    #[napi(js_name = "toMatlab")]
    pub fn to_matlab(&self) -> Result<String> {
        Ok(self.session.render_as_matlab(self.root))
    }

    /// Structural equality (Form round-trip compare).
    #[napi(js_name = "isEqual")]
    pub fn is_equal(&self, other: &Expression) -> Result<bool> {
        Ok(self.session.to_mathematica(self.root) == other.session.to_mathematica(other.root))
    }

    /// Render 1-D `Plot` / `plot` as SVG when the term matches a known form.
    #[napi(js_name = "plotSvg")]
    pub fn plot_svg(&self) -> Result<String> {
        match self.session.try_plot_svg(self.root, self.dialect) {
            Some(Ok(svg)) => Ok(svg),
            Some(Err(e)) => Err(map_err(e)),
            None => Err(Error::from_reason("not a supported 1-D plot form")),
        }
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
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, &input, d)?;
    let root = session.differentiate_term(term, &var);
    Ok(Expression { session, root, dialect: resolved })
}

/// Top-level `simplify(expr, dialect?)`.
#[napi]
pub fn simplify(input: String, dialect: Option<String>) -> Result<Expression> {
    let d = dialect_from_str(dialect)?;
    let session = Session::new();
    let (term, resolved) = parse_to_term(&session, &input, d)?;
    let evaluated = session.evaluate(term);
    let root = session.simplify_term(evaluated);
    Ok(Expression { session, root, dialect: resolved })
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
