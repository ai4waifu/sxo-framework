//! Simple Math parse (off current delivery route).

use sxo_types::SxoError;

use crate::form::Expr;

/// Parse simple-math text into flat [`Expr`].
pub fn parse(_input: &str) -> Result<Expr, SxoError> {
    Err(SxoError::new("simple-math: parse not on delivery route"))
}
