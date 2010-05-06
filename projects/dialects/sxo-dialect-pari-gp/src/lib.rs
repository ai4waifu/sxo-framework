//! PARI/GP dialect scaffold: [`GpForm`], stub parse / lower / render.
//!
//! `oak-pari` and Athena domain contracts are not wired yet. Hosts must keep
//! this dialect unsupported until Feature Matrix cases promote off `planned`/`gap`.

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod render;

pub use form::{GpAtom, GpForm};
pub use lower::lower_request;
pub use parse::parse_gp_form;
pub use render::render_gp;
