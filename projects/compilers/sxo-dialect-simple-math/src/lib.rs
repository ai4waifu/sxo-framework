//! Simple Math dialect: flat Form, lower, render.

#![deny(missing_docs)]

mod form;
mod lower;
mod parse;
mod render;

pub use form::Expr;
pub use lower::{expr_from_session, lower_expr};
pub use parse::parse as parse_simple_math;
pub use render::render;
