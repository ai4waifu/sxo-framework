use num_bigint::BigInt;
use sxo_dialects::{Number, Term, WExpr, evaluate, parse_mathematica, render_wexpr, term_to_wexpr, wexpr_to_term};

#[test]
fn parse_plus_times() {
    let w = parse_mathematica("1 + 2 * 3").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::int(7));
}

#[test]
fn parse_list() {
    let w = parse_mathematica("{1, 2 + 2}").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::List(vec![Term::int(1), Term::int(4)]));
}

#[test]
fn parse_power_one() {
    let w = parse_mathematica("Power[x, 1]").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::symbol("x"));
}

#[test]
fn parse_sin() {
    let e = parse_mathematica("Sin[x]").unwrap();
    assert_eq!(e, WExpr::call("Sin", vec![WExpr::symbol("x")]));
}

#[test]
fn parse_d() {
    let w = parse_mathematica("D[x^3, x]").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    let s = render_wexpr(&term_to_wexpr(&e));
    assert!(s.contains('x'), "got {s}");
}

#[test]
fn parse_compound_expression() {
    let w = parse_mathematica("a; b").unwrap();
    assert_eq!(w, WExpr::call("CompoundExpression", vec![WExpr::symbol("a"), WExpr::symbol("b")]));
}

#[test]
fn parse_root_semicolon_returns_last() {
    let w = parse_mathematica("1 + 2; 3 * 4").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::int(12));
}

#[test]
fn parse_equal_and_factorial() {
    let w = parse_mathematica("2 == 2").unwrap();
    assert_eq!(w, WExpr::call("Equal", vec![WExpr::int(2), WExpr::int(2)]));
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::int(1));

    let w = parse_mathematica("5!").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::int(120));
}

#[test]
fn parse_big_integer() {
    let w = parse_mathematica("99999999999999999999").unwrap();
    assert_eq!(w, WExpr::number(Number::integer(BigInt::parse_bytes(b"99999999999999999999", 10).unwrap())));
}

#[test]
fn parse_map_and_integrate() {
    let w = parse_mathematica("Map[Sin, {0, Pi/2}]").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert!(matches!(e, Term::List(_)));

    let w = parse_mathematica("Integrate[x^2, x]").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    let s = render_wexpr(&term_to_wexpr(&e));
    assert!(s.contains('x'), "got {s}");
}
