//! PARI/GP parse stub — waits for `oak-pari`.

use sxo_types::SxoError;

use crate::form::GpForm;

/// Parse GP text into [`GpForm`].
///
/// Always errors until `oak-pari` lands. Do not hand-write a second parser here.
pub fn parse_gp_form(_input: &str) -> Result<GpForm, SxoError> {
    Err(SxoError::new("pari-gp: parse not implemented (needs oak-pari → GpForm)"))
}
