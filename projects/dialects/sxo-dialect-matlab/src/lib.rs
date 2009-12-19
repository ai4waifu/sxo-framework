//! MATLAB dialect: oak parse, lower, render (no `WExpr`).

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod plot;
mod render;
mod shared;
mod surface;

pub use lower::lower_request;
pub use parse::parse_matlab;
pub use plot::try_plot_svg;
pub use render::render_matlab;
pub use shared::{parse_number_literal, render_number};
pub use surface::{application_surface_name, push_matlab_call, semantic_to_surface, surface_to_semantic};
