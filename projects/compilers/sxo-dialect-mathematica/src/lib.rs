//! Mathematica / Wolfram dialect: `WExpr`, oak parse, lower, render.

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod plot;
mod render;
mod shared;

pub use form::{WAtom, WExpr};
pub use lower::{lower_request, lower_wexpr, push_surface_call, semantic_to_surface, surface_to_semantic, wexpr_from_session};
pub use parse::parse_mathematica;
pub use plot::try_plot_svg;
pub use render::render;
pub use shared::{parse_number_literal, render_number};
