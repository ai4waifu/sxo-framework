//! Simple Math dialect: flat Form, lower, render.

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod render;

pub use form::Expr;
pub use lower::{lower_expr, expr_from_session};
pub use parse::parse as parse_simple_math;
pub use render::render;
