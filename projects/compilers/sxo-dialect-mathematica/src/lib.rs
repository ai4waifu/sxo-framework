//! Mathematica / Wolfram dialect: `WExpr`, oak parse, lower, render.

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod plot;
mod render;
mod shared;

pub use form::{WAtom, WExpr};
pub use lower::{term_to_wexpr, wexpr_to_term};
pub use parse::parse_mathematica;
pub use plot::try_plot_svg;
pub use render::render;
pub use shared::{parse_number_literal, render_number, term_from_number_literal};
