//! Explicit dialect tags for the N-API surface (no auto detection).

use napi::bindgen_prelude::*;
use sxo_types::{Dialect, SxoError};

use crate::session::Session;
use athena::types::TermId;

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

pub(crate) fn parse_to_term(session: &Session, input: &str, dialect: Dialect) -> Result<(TermId, Dialect)> {
    let term = match dialect {
        Dialect::Mathematica => {
            let w = session.parse_mathematica(input).map_err(map_err)?;
            session.lower_mathematica(&w)
        }
        Dialect::Matlab => session.parse_matlab(input).map_err(map_err)?,
        Dialect::SimpleMath => {
            return Err(Error::from_reason("simple-math dialect is off the current delivery route"));
        }
    };
    Ok((term, dialect))
}
