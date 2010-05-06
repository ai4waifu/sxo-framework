//! [`GpForm`] → Athena request stub.

use sxo_types::SxoError;

use crate::form::GpForm;

/// Lower a [`GpForm`] into an Athena request.
///
/// Always errors until domain contracts and typed lowering land.
/// Hosts must not invent a placeholder evaluate path around this.
pub fn lower_request(_form: &GpForm) -> Result<(), SxoError> {
    Err(SxoError::new(
        "pari-gp: GpForm lowering not implemented (Athena domain contracts pending)",
    ))
}
