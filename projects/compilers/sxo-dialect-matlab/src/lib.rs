//! MATLAB dialect: oak parse, lower, render (no `WExpr`).

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod plot;
mod render;
mod shared;

pub use lower::lower_request;
pub use parse::parse_matlab;
pub use plot::try_plot_svg;
pub use render::render_matlab;
pub use shared::{parse_number_literal, render_number};
