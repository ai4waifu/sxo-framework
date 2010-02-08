//! MATLAB dialect: oak parse, lower, render (no `WExpr`).

#![deny(missing_docs)]

mod form;
mod lower;
mod number_literal;
mod parse;
mod plot;
mod render;
mod surface;

pub use form::{MatlabAtom, MatlabForm};
pub use lower::{form_to_term, lower_request, lower_term_request};
pub use number_literal::{parse_number_literal, render_number};
pub use parse::{parse_matlab, parse_matlab_form};
pub use plot::try_plot_svg;
pub use render::render_matlab;
pub use surface::{application_surface_name, push_matlab_call, semantic_to_surface, surface_to_semantic};
