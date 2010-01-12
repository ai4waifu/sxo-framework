//! Explicit dialect tags for the WASM surface (no auto detection).

use sxo_types::{Dialect, SxoError};
use wasm_bindgen::prelude::*;

use crate::session::Session;
use athena::types::TermId;

pub(crate) fn map_err(err: SxoError) -> JsValue {
    JsValue::from_str(&err.message)
}

pub(crate) fn dialect_from_str(s: Option<String>) -> Result<Dialect, JsValue> {
    match s.as_deref() {
        None => Err(JsValue::from_str("dialect is required (mathematica | matlab | simple-math)")),
        Some("auto") => Err(JsValue::from_str("dialect auto is removed. pass an explicit dialect")),
        Some("simple-math") | Some("sm") => Ok(Dialect::SimpleMath),
        Some("mathematica") => Ok(Dialect::Mathematica),
        Some("matlab") => Ok(Dialect::Matlab),
        Some(other) => Err(JsValue::from_str(&format!("unknown dialect: {other}"))),
    }
}

pub(crate) fn parse_to_term(session: &Session, input: &str, dialect: Dialect) -> Result<(TermId, Dialect), JsValue> {
    let term = match dialect {
        Dialect::Mathematica => {
            let w = session.parse_mathematica(input).map_err(map_err)?;
            session.lower_mathematica(&w)
        }
        Dialect::Matlab => session.parse_matlab(input).map_err(map_err)?,
        Dialect::SimpleMath => {
            return Err(JsValue::from_str("simple-math dialect is off the current delivery route"));
        }
    };
    Ok((term, dialect))
}
