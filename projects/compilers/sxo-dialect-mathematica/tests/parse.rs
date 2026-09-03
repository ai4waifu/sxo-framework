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
    assert_eq!(e, Term::boolean(true));

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

#[test]
fn parse_true_false_null_atoms() {
    assert_eq!(parse_mathematica("True").unwrap(), WExpr::symbol("True"));
    assert_eq!(parse_mathematica("False").unwrap(), WExpr::symbol("False"));
    assert_eq!(parse_mathematica("Null").unwrap(), WExpr::symbol("Null"));
    assert_eq!(evaluate(&wexpr_to_term(&parse_mathematica("True").unwrap())), Term::boolean(true));
    assert_eq!(evaluate(&wexpr_to_term(&parse_mathematica("False").unwrap())), Term::boolean(false));
    assert_eq!(evaluate(&wexpr_to_term(&parse_mathematica("Null").unwrap())), Term::null());
    assert_eq!(render(&term_to_wexpr(&Term::boolean(true))), "True");
    assert_eq!(render(&term_to_wexpr(&Term::null())), "Null");
}

#[test]
fn parse_and_or_not_bool_atoms() {
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("And[True, False]").unwrap())),
        Term::boolean(false)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Or[False, True]").unwrap())),
        Term::boolean(true)
    );
    assert_eq!(evaluate(&wexpr_to_term(&parse_mathematica("Not[True]").unwrap())), Term::boolean(false));
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Which[False, 1, True, 2]").unwrap())),
        Term::int(2)
    );
    assert_eq!(evaluate(&wexpr_to_term(&parse_mathematica("1 == 1").unwrap())), Term::boolean(true));
}

#[test]
fn parse_with_module_block_local_bindings() {
    for src in ["With[{x = 1}, x + 1]", "Module[{x = 1}, x + 1]", "Block[{x = 1}, x + 1]"] {
        let e = evaluate(&wexpr_to_term(&parse_mathematica(src).unwrap()));
        assert_eq!(e, Term::int(2), "{src}");
    }
}

#[test]
fn parse_slot_lowers_to_slot_head() {
    let w = parse_mathematica("#").unwrap();
    assert_eq!(w, WExpr::call("Slot", vec![WExpr::int(1)]));
}

#[test]
fn parse_pure_function_slot_application() {
    let e = evaluate(&wexpr_to_term(&parse_mathematica("(#^2)&[4]").unwrap()));
    assert_eq!(e, Term::int(16));
}

#[test]
fn parse_named_function_application() {
    let e = evaluate(&wexpr_to_term(&parse_mathematica("Function[x, x^2][3]").unwrap()));
    assert_eq!(e, Term::int(9));
}

#[test]
fn parse_map_pure_function() {
    let e = evaluate(&wexpr_to_term(&parse_mathematica("Map[#^2 &, {1, 2, 3}]").unwrap()));
    assert_eq!(e, Term::List(vec![Term::int(1), Term::int(4), Term::int(9)]));
}

#[test]
fn parse_blank_and_typed_blank() {
    assert_eq!(parse_mathematica("_").unwrap(), WExpr::call("Blank", vec![]));
    assert_eq!(
        parse_mathematica("_Integer").unwrap(),
        WExpr::call("Blank", vec![WExpr::symbol("Integer")])
    );
    assert_eq!(
        parse_mathematica("x_").unwrap(),
        WExpr::call("Pattern", vec![WExpr::symbol("x"), WExpr::call("Blank", vec![])])
    );
}

#[test]
fn parse_match_q_and_cases() {
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("MatchQ[1, _Integer]").unwrap())),
        Term::boolean(true)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("MatchQ[a, _Integer]").unwrap())),
        Term::boolean(false)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Cases[{1, a, 2}, _Integer]").unwrap())),
        Term::List(vec![Term::int(1), Term::int(2)])
    );
}

#[test]
fn parse_table_range_apply_list_primitives() {
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Table[i, {i, 3}]").unwrap())),
        Term::List(vec![Term::int(1), Term::int(2), Term::int(3)])
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Range[3]").unwrap())),
        Term::List(vec![Term::int(1), Term::int(2), Term::int(3)])
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Apply[Plus, {1, 2, 3}]").unwrap())),
        Term::int(6)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Length[{1, 2, 3}]").unwrap())),
        Term::int(3)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Join[{1}, {2}]").unwrap())),
        Term::List(vec![Term::int(1), Term::int(2)])
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("First[{a, b}]").unwrap())),
        Term::symbol("a")
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Sum[i, {i, 1, 10}]").unwrap())),
        Term::int(55)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Product[i, {i, 1, 5}]").unwrap())),
        Term::int(120)
    );
}

#[test]
fn parse_limit_sinc_and_definite_integrate_sin() {
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Limit[Sin[x]/x, x -> 0]").unwrap())),
        Term::int(1)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Integrate[Sin[x], {x, 0, Pi}]").unwrap())),
        Term::int(2)
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Cos[Pi]").unwrap())),
        Term::int(-1)
    );
}

#[test]
fn parse_linear_solve_nested_lists() {
    assert_eq!(
        evaluate(&wexpr_to_term(
            &parse_mathematica("LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]").unwrap()
        )),
        Term::List(vec![
            Term::List(vec![Term::int(-4)]),
            Term::List(vec![Term::rational_i64(9, 2).unwrap()]),
        ])
    );
    assert_eq!(
        evaluate(&wexpr_to_term(&parse_mathematica("Det[{{1, 2}, {3, 4}}]").unwrap())),
        Term::int(-2)
    );
}

#[test]
fn parse_solve_quadratic_x2_eq_1() {
    let e = evaluate(&wexpr_to_term(&parse_mathematica("Solve[x^2 == 1, x]").unwrap()));
    assert_eq!(
        e,
        Term::List(vec![
            Term::List(vec![Term::apply("Rule", vec![Term::symbol("x"), Term::int(-1)])]),
            Term::List(vec![Term::apply("Rule", vec![Term::symbol("x"), Term::int(1)])]),
        ])
    );
    assert_eq!(render(&term_to_wexpr(&e)), "{{x -> -1}, {x -> 1}}");
}
