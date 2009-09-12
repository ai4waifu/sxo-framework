//! Integration tests for frontend.

use num_bigint::BigInt;
use sxo_dialects::{
    AssumptionSet, CalculusRequest, CalculusResult, CalculusValue, DerivativeOrder, DomainRequest, DomainResult, LimitApproach,
    LimitDirection, Remainder, SxoFrontend, Term, TermKind, WExpr, lower_calculus_term, term_to_wexpr, wexpr_to_term,
};

#[test]
fn math_evaluate_arith() {
    let eng = SxoFrontend::new();
    let e = eng.evaluate_mathematica("1 + 2 * 3").unwrap();
    assert_eq!(e, Term::int(7));
}

#[test]
fn math_d_form() {
    let eng = SxoFrontend::new();
    let e = eng.d_mathematica("x^3", "x").unwrap();
    let s = eng.render_as_wolfram(&e);
    assert!(s.contains('x'), "got {s}");
}

#[test]
fn math_simplify_trig() {
    let eng = SxoFrontend::new();
    let w = eng.parse_mathematica("Simplify[Sin[x]^2 + Cos[x]^2]").unwrap();
    let e = eng.evaluate(&eng.from_mathematica(&w));
    assert_eq!(e, Term::int(1));
}

#[test]
fn matlab_integrate_matrix() {
    let eng = SxoFrontend::new();
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
    assert_eq!(t, Term::apply("Sin", vec![Term::symbol("x")]));
    assert_eq!(term_to_wexpr(&t), w);
}

#[test]
fn big_integer_arithmetic() {
    let eng = SxoFrontend::new();
    let e = eng.evaluate_mathematica("99999999999999999999 + 1").unwrap();
    assert_eq!(e, Term::integer(BigInt::parse_bytes(b"100000000000000000000", 10).unwrap()));
    let div = eng.evaluate_mathematica("1/3 + 1/3 + 1/3").unwrap();
    assert_eq!(div, Term::int(1));
}

#[test]
fn bridge_lowers_to_kernel() {
    let eng = SxoFrontend::new();
    let w = eng.parse_mathematica("1 + 2").unwrap();
    let t = eng.from_mathematica(&w);
    let k = eng.lower_to_kernel(&t).unwrap();
    assert!(matches!(k.arena.get(k.root), Some(TermKind::App { .. })));
}

#[test]
fn athena_domain_derivative_and_definite_integral() {
    let eng = SxoFrontend::new();
    let expr = eng.from_mathematica(&eng.parse_mathematica("x^3").unwrap());
    let d = eng
        .execute_domain(DomainRequest::Calculus(CalculusRequest::Derivative {
            expression: expr,
            variable: "x".into(),
            order: DerivativeOrder::First,
            assumptions: AssumptionSet::empty(),
        }))
        .expect("ok");
    match d {
        DomainResult::Calculus(CalculusResult::Exact { value: CalculusValue::Expression(v), .. }) => {
            let s = eng.render_as_wolfram(&v);
            assert!(s.contains('x'), "got {s}");
        }
        other => panic!("expected Exact expression, got {other:?}"),
    }

    let integ = eng
        .execute_domain(DomainRequest::Calculus(CalculusRequest::DefiniteIntegral {
            expression: Term::symbol("x"),
            variable: "x".into(),
            lower: Term::int(0),
            upper: Term::int(2),
            assumptions: AssumptionSet::empty(),
        }))
        .expect("ok");
    match integ {
        DomainResult::Calculus(CalculusResult::Exact { value: CalculusValue::Expression(v), .. }) => {
            assert_eq!(v, Term::int(2))
        }
        other => panic!("expected Exact 2, got {other:?}"),
    }
}

#[test]
fn athena_domain_limit_and_series() {
    let eng = SxoFrontend::new();
    let lim = eng
        .execute_domain(DomainRequest::Calculus(CalculusRequest::Limit {
            expression: Term::apply("Plus", vec![Term::apply("Power", vec![Term::symbol("x"), Term::int(2)]), Term::int(1)]),
            variable: "x".into(),
            approach: LimitApproach::Finite(Term::int(2)),
            direction: LimitDirection::TwoSided,
            assumptions: AssumptionSet::empty(),
        }))
        .expect("ok");
    match lim {
        DomainResult::Calculus(CalculusResult::Exact { value: CalculusValue::Expression(v), .. }) => {
            assert_eq!(v, Term::int(5))
        }
        other => panic!("expected Exact 5, got {other:?}"),
    }

    let series = eng
        .execute_domain(DomainRequest::Calculus(CalculusRequest::Series {
            expression: Term::apply("Power", vec![Term::symbol("x"), Term::int(2)]),
            variable: "x".into(),
            center: Term::int(0),
            order: 3,
            assumptions: AssumptionSet::empty(),
        }))
        .expect("ok");
    match series {
        DomainResult::Calculus(CalculusResult::Exact { value: CalculusValue::Series(s), .. }) => {
            assert_eq!(s.remainder, Remainder::ExactTruncation)
        }
        other => panic!("expected Exact Series, got {other:?}"),
    }
}

#[test]
fn dialect_d_limit_series_lower_to_domain() {
    let eng = SxoFrontend::new();

    let d_term = eng.from_mathematica(&eng.parse_mathematica("D[x^3, x]").unwrap());
    assert!(matches!(lower_calculus_term(&d_term), Some(DomainRequest::Calculus(CalculusRequest::Derivative { .. }))));
    let d_out = eng.evaluate(&d_term);
    let d_s = eng.render_as_wolfram(&d_out);
    assert!(d_s.contains('x'), "got {d_s}");

    let lim = Term::apply(
        "Limit",
        vec![
            Term::apply("Plus", vec![Term::apply("Power", vec![Term::symbol("x"), Term::int(2)]), Term::int(1)]),
            Term::apply("Rule", vec![Term::symbol("x"), Term::int(2)]),
        ],
    );
    assert!(matches!(lower_calculus_term(&lim), Some(DomainRequest::Calculus(CalculusRequest::Limit { .. }))));
    assert_eq!(eng.evaluate(&lim), Term::int(5));

    let series = Term::apply(
        "Series",
        vec![
            Term::apply("Power", vec![Term::symbol("x"), Term::int(2)]),
            Term::List(vec![Term::symbol("x"), Term::int(0), Term::int(3)]),
        ],
    );
    assert!(matches!(lower_calculus_term(&series), Some(DomainRequest::Calculus(CalculusRequest::Series { .. }))));
    let s_out = eng.evaluate(&series);
    let text = format!("{s_out:?}");
    assert!(text.contains('x'), "got {text}");
}
