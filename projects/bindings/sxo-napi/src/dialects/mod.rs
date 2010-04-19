//! Explicit dialect tags for the N-API surface (no auto detection).

use napi::bindgen_prelude::*;
use sxo_dialect_mathematica::WolframForm;
use sxo_dialect_matlab::MatlabForm;
use sxo_types::{Dialect, SxoError};

use crate::session::Session;
use athena::types::TermId;

/// Dialect Form retained on parse objects (Living 17 / R-2.11).
#[derive(Debug, Clone)]
pub(crate) enum HeldForm {
    /// MATLAB surface Form.
    Matlab(MatlabForm),
    /// Mathematica / Wolfram surface Form.
    Wolfram(WolframForm),
}

pub(crate) fn map_err(err: SxoError) -> Error {
    Error::from_reason(err.message)
}

pub(crate) fn dialect_from_str(s: Option<String>) -> Result<Dialect> {
    match s.as_deref() {
        None => Err(Error::from_reason("dialect is required (mathematica | matlab | simple-math)")),
        Some("auto") => Err(Error::from_reason("dialect auto is removed. pass an explicit dialect")),
        Some("simple-math") | Some("sm") => Ok(Dialect::SimpleMath),
        Some("mathematica") => Ok(Dialect::Mathematica),
        Some("matlab") => Ok(Dialect::Matlab),
        Some(other) => Err(Error::from_reason(format!("unknown dialect: {other}"))),
    }
}

pub(crate) fn dialect_to_str(d: Dialect) -> &'static str {
    match d {
        Dialect::SimpleMath => "simple-math",
        Dialect::Mathematica => "mathematica",
        Dialect::Matlab => "matlab",
    }
}

/// Parse source into arena root + retained dialect Form.
pub(crate) fn parse_held(session: &Session, input: &str, dialect: Dialect) -> Result<(TermId, HeldForm, Dialect)> {
    match dialect {
        Dialect::Mathematica => {
            let w = session.parse_mathematica(input).map_err(map_err)?;
            let term = session.lower_mathematica(&w);
            Ok((term, HeldForm::Wolfram(w), dialect))
        }
        Dialect::Matlab => {
            let form = session.parse_matlab_form(input).map_err(map_err)?;
            let term = session.lower_matlab(&form);
            Ok((term, HeldForm::Matlab(form), dialect))
        }
        Dialect::SimpleMath => Err(Error::from_reason("simple-math dialect is off the current delivery route")),
    }
}

/// Materialize only (plot / d entry helpers). Prefer [`parse_held`] for Expression handles.
pub(crate) fn parse_to_term(session: &Session, input: &str, dialect: Dialect) -> Result<(TermId, Dialect)> {
    let (term, _, resolved) = parse_held(session, input, dialect)?;
    Ok((term, resolved))
}
