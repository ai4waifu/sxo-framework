//! Integration tests for Mathematica parse.

use athena::{Term, evaluate};
use sxo_dialect_mathematica::{WAtom, WExpr, parse_mathematica, parse_number_literal, render, term_to_wexpr, wexpr_to_term};

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
    let s = render(&term_to_wexpr(&e));
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
    let n = parse_number_literal("99999999999999999999").unwrap();
    let w = parse_mathematica("99999999999999999999").unwrap();
    assert_eq!(w, WExpr::number(n));
}

#[test]
fn parse_if_call_shape() {
    let w = parse_mathematica("If[1==1,7,8]").unwrap();
    assert_eq!(
        w,
        WExpr::call(
            "If",
            vec![
                WExpr::call("Equal", vec![WExpr::int(1), WExpr::int(1)]),
                WExpr::int(7),
                WExpr::int(8),
            ]
        )
    );
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::int(7));
}

#[test]
fn parse_hold_keeps_args() {
    let w = parse_mathematica("Hold[1+1]").unwrap();
    assert_eq!(w, WExpr::call("Hold", vec![WExpr::call("Plus", vec![WExpr::int(1), WExpr::int(1)])]));
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::apply("Hold", vec![Term::apply("Plus", vec![Term::int(1), Term::int(1)])]));
}

#[test]
fn parse_hold_form_keeps_args() {
    let w = parse_mathematica("HoldForm[1+1]").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::apply("HoldForm", vec![Term::apply("Plus", vec![Term::int(1), Term::int(1)])]));
}

#[test]
fn parse_import_call_shape() {
    let w = parse_mathematica("Import[\"x.csv\"]").unwrap();
    assert_eq!(w, WExpr::call("Import", vec![WExpr::Atom(WAtom::String("x.csv".into()))]));
}

#[test]
fn parse_part_double_bracket() {
    let w = parse_mathematica("{1,2,3}[[0]]").unwrap();
    assert_eq!(
        w,
        WExpr::call(
            "Part",
            vec![WExpr::List(vec![WExpr::int(1), WExpr::int(2), WExpr::int(3)]), WExpr::int(0)]
        )
    );
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::symbol("List"));
}

#[test]
fn parse_part_call_zero() {
    let w = parse_mathematica("Part[{1,2,3},0]").unwrap();
    let e = evaluate(&wexpr_to_term(&w));
    assert_eq!(e, Term::symbol("List"));
}
