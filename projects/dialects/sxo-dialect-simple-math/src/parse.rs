//! Simple Math parse stub — waits for `oak-athena`.

use sxo_types::SxoError;

use crate::form::Expr;

/// Parse Simple Math text into flat [`Expr`].
///
/// Always errors until `oak-athena` lands. Do not hand-write a second parser here.
pub fn parse(_input: &str) -> Result<Expr, SxoError> {
    Err(SxoError::new("simple-math: parse not on delivery route (needs oak-athena → Expr)"))
}
