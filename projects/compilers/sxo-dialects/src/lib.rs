//! SXO dialect layer: oak parse, Form, lowering, render (no math engine).
//!
//! | Layer | Owner | Types |
//! |-------|-------|-------|
//! | Product | `sxo-types` | `Dialect`, `SxoError` |
//! | Math | `athena` | `Number`, `Term`, `AthenaEngine`, arena IR |
//! | Dialect | this crate | `WExpr`, parse/render, `SxoFrontend` |

#![deny(missing_docs)]

mod bridge;
mod diff;
mod domain_lower;
mod engine;
mod expr;
mod lowering;
mod mma_bridge;
mod number_literal;
mod parse_math;
mod parse_matlab;
mod render;
mod render_matlab;
mod render_wexpr;
mod simplify;
mod wexpr;

pub use athena::{
    AssumptionSet, AthenaEngine, Atom, CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder, Diagnostic,
    DiagnosticCode, DomainRequest, DomainResult, ExactNumber, LimitApproach, LimitDirection, Number, RealNumber, Remainder,
    Series, Term, TermArena, TermBuilder, TermId, TermKind, differentiate_term, evaluate, number_from_term,
    try_calculus_request,
};
pub use domain_lower::{derivative_request, limit_request, lower_calculus_term, series_request, try_evaluate_calculus};
pub use engine::SxoFrontend;
pub use expr::Expr;
pub use lowering::{KernelTerm, lower_to_kernel};
pub use mma_bridge::{term_to_wexpr, wexpr_to_term};
pub use number_literal::{parse_number_literal, render_number, term_from_number_literal};
pub use parse_math::parse_mathematica;
pub use parse_matlab::parse_matlab;
pub use render::{render_mathematica, render_simple_math};
pub use render_matlab::render_matlab;
pub use render_wexpr::render_wexpr;
pub use sxo_types::{Dialect, Severity, SourceSpan, SxoError, VERSION, detect_dialect};
pub use wexpr::{WAtom, WExpr};
