//! Opaque N-API expression handles.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use sxo_types::Dialect;

use crate::dialects::{dialect_from_str, dialect_to_str, map_err, parse_to_term};
use crate::session::Session;
use athena::types::TermId;

/// Opaque expression handle backed by a host [`Session`] arena [`TermId`].
#[derive(Debug)]
#[napi]
pub struct Expression {
    pub(crate) session: Session,
    pub(crate) root: TermId,
    pub(crate) dialect: Dialect,
}

/// Fork `root` into a fresh host [`Session`] via dialect-native round-trip.
///
/// MATLAB must **not** go through Mathematica Form — that rewrites surface heads
/// (`diff` / `eye` / …) and breaks MATLAB `lower_request`.
fn fork_expression(session: &Session, root: TermId, dialect: Dialect) -> Result<Expression> {
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

#[napi]
impl Expression {
    /// Parse `input` with an explicit dialect (`mathematica` | `matlab` | `simple-math`).
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
        let mut out = fork_expression(&self.session, self.root, self.dialect)?;
        out.root = out.session.differentiate_term(out.root, &var);
        Ok(out)
    }

    /// Simplify via the engine (`Simplify` head).
    #[napi]
    pub fn simplify(&self) -> Result<Expression> {
        let mut out = fork_expression(&self.session, self.root, self.dialect)?;
        out.root = out.session.simplify_term(out.root);
        Ok(out)
    }

    /// Evaluate via dialect `lower_request` (Session / Control / Domain Goals).
    #[napi]
    pub fn evaluate(&self) -> Result<Expression> {
        let mut out = fork_expression(&self.session, self.root, self.dialect)?;
        out.root = out.session.evaluate_form(out.root, out.dialect).map_err(map_err)?;
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
