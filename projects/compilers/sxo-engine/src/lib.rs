//! SXO bridge: oak parse, frontend forms, lowering to Euler kernel IR.
//!
//! | Layer | Owner | Types |
//! |-------|-------|-------|
//! | Product | `sxo-types` | `Dialect`, `SxoError` |
//! | Kernel | `euler` | `Number`, `TermArena`, `TermId`, eval (future) |
//! | Bridge | this crate | `Term`, `WExpr`, parse/render, `weval` (transition) |

#![deny(missing_docs)]

mod bridge;
mod diff;
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
mod term;
mod weval;
mod wexpr;

pub use engine::SxoEngine;
pub use euler::{
    Diagnostic, DiagnosticCode, ExactNumber, Number, RealNumber, TermArena, TermBuilder, TermId,
    TermKind,
};
pub use number_literal::{parse_number_literal, render_number};
pub use expr::Expr;
pub use lowering::{KernelTerm, lower_to_kernel};
pub use mma_bridge::{term_to_wexpr, wexpr_to_term};
pub use parse_math::parse_mathematica;
pub use parse_matlab::parse_matlab;
pub use render::{render_mathematica, render_simple_math};
pub use render_matlab::render_matlab;
pub use render_wexpr::render_wexpr;
pub use sxo_types::{Dialect, SxoError, Severity, SourceSpan, VERSION, detect_dialect};
pub use term::{Atom, Term, number_from_term};
pub use weval::{differentiate as differentiate_term, evaluate};
pub use wexpr::{WAtom, WExpr};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_evaluate_arith() {
        let eng = SxoEngine::new();
        let e = eng.evaluate_mathematica("1 + 2 * 3").unwrap();
        assert_eq!(e, Term::int(7));
    }

    #[test]
    fn math_d_form() {
        let eng = SxoEngine::new();
        let e = eng.d_mathematica("x^3", "x").unwrap();
        let s = eng.render_as_wolfram(&e);
        assert!(s.contains('x'), "got {s}");
    }

    #[test]
    fn math_simplify_trig() {
        let eng = SxoEngine::new();
        let w = eng.parse_mathematica("Simplify[Sin[x]^2 + Cos[x]^2]").unwrap();
        let e = eng.evaluate(&eng.from_mathematica(&w));
        assert_eq!(e, Term::int(1));
    }

    #[test]
    fn matlab_integrate_matrix() {
        let eng = SxoEngine::new();
        let t = eng.parse_matlab("int(x^2, x)").unwrap();
        let e = eng.evaluate(&t);
        assert!(eng.render_as_matlab(&e).contains('x'));

        let m = eng.parse_matlab("[1, 2; 3, 4]").unwrap();
        assert_eq!(eng.render_as_matlab(&m), "[1, 2; 3, 4]");
    }

    #[test]
    fn wexpr_is_not_term() {
        let w = WExpr::call("Sin", vec![WExpr::symbol("x")]);
        let t = wexpr_to_term(&w);
        assert_eq!(t, Term::app("Sin", vec![Term::symbol("x")]));
        assert_eq!(term_to_wexpr(&t), w);
    }

    #[test]
    fn big_integer_arithmetic() {
        use num_bigint::BigInt;
        let eng = SxoEngine::new();
        let e = eng
            .evaluate_mathematica("99999999999999999999 + 1")
            .unwrap();
        assert_eq!(e, Term::integer(BigInt::parse_bytes(b"100000000000000000000", 10).unwrap()));
        let div = eng.evaluate_mathematica("1/3 + 1/3 + 1/3").unwrap();
        assert_eq!(div, Term::int(1));
    }

    #[test]
    fn bridge_lowers_to_kernel() {
        let eng = SxoEngine::new();
        let w = eng.parse_mathematica("1 + 2").unwrap();
        let t = eng.from_mathematica(&w);
        let k = eng.lower_to_kernel(&t).unwrap();
        assert!(matches!(k.arena.get(k.root), Some(TermKind::App { .. })));
    }
}
