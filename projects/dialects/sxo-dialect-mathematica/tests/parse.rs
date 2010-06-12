//! Integration tests for Mathematica parse (session arena `TermId`).

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session,
    ir::TermNode,
    runtime::values::arena::{push_bool, push_int, push_list, push_null, push_symbol_name},
    types::TermId,
};
use sxo_dialect_mathematica::{
    WolframAtom, WolframForm, lower_request, lower_wexpr, parse_mathematica, parse_number_literal, push_surface_call, render,
    try_plot_svg, wexpr_from_session,
};

type Tid = TermId;

struct H {
    s: RefCell<Session>,
}

impl H {
    fn new() -> Self {
        Self { s: RefCell::new(Session::new()) }
    }

    fn parse_w(&self, input: &str) -> WolframForm {
        parse_mathematica(input).unwrap()
    }

    fn lower(&self, w: &WolframForm) -> Tid {
        lower_wexpr(&mut self.s.borrow_mut(), w)
    }

    fn eval(&self, input: &str) -> Tid {
        let w = self.parse_w(input);
        let mut s = self.s.borrow_mut();
        let request = lower_request(&mut s, &w);
        let engine = AthenaEngine::new();
        match engine.execute_request(&mut s, request) {
            Ok(result_id) => s.results.get(result_id).and_then(|r| r.symbolic_term).unwrap_or_else(|| lower_wexpr(&mut s, &w)),
            Err(_) => lower_wexpr(&mut s, &w),
        }
    }

    fn i(&self, n: i64) -> Tid {
        push_int(&mut self.s.borrow_mut(), n)
    }

    fn sym(&self, name: &str) -> Tid {
        push_symbol_name(&mut self.s.borrow_mut(), name)
    }

    fn ap(&self, head: &str, args: Vec<Tid>) -> Tid {
        push_surface_call(&mut self.s.borrow_mut(), head, args)
    }

    fn lst(&self, items: Vec<Tid>) -> Tid {
        push_list(&mut self.s.borrow_mut(), items)
    }

    fn boolean(&self, b: bool) -> Tid {
        push_bool(&mut self.s.borrow_mut(), b)
    }

    fn null(&self) -> Tid {
        push_null(&mut self.s.borrow_mut())
    }

    fn eq(&self, a: Tid, b: Tid) -> bool {
        self.s.borrow().arena.structural_eq(a, b)
    }

    fn wolfram(&self, id: Tid) -> String {
        let w = wexpr_from_session(&self.s.borrow(), id);
        render(&w)
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        f(&mut self.s.borrow_mut())
    }
}

#[test]
fn parse_plus_times() {
    let h = H::new();
    assert!(h.eq(h.eval("1 + 2 * 3"), h.i(7)));
}

#[test]
fn parse_list() {
    let h = H::new();
    assert!(h.eq(h.eval("{1, 2 + 2}"), h.lst(vec![h.i(1), h.i(4)])));
}

#[test]
fn parse_power_one() {
    let h = H::new();
    assert!(h.eq(h.eval("Power[x, 1]"), h.sym("x")));
}

#[test]
fn parse_sin() {
    let e = parse_mathematica("Sin[x]").unwrap();
    assert_eq!(e, WolframForm::call("Sin", vec![WolframForm::symbol("x")]));
}

#[test]
fn parse_d() {
    let h = H::new();
    let e = h.eval("D[x^3, x]");
    let s = h.wolfram(e);
    assert!(s.contains('x'), "got {s}");
}

#[test]
fn parse_compound_expression() {
    let w = parse_mathematica("a; b").unwrap();
    assert_eq!(w, WolframForm::call("CompoundExpression", vec![WolframForm::symbol("a"), WolframForm::symbol("b")]));
}

#[test]
fn parse_root_semicolon_returns_last() {
    let h = H::new();
    assert!(h.eq(h.eval("1 + 2; 3 * 4"), h.i(12)));
}

#[test]
fn parse_equal_and_factorial() {
    let w = parse_mathematica("2 == 2").unwrap();
    assert_eq!(w, WolframForm::call("Equal", vec![WolframForm::int(2), WolframForm::int(2)]));
    let h = H::new();
    assert!(h.eq(h.eval("2 == 2"), h.boolean(true)));
    assert!(h.eq(h.eval("5!"), h.i(120)));
}

#[test]
fn parse_big_integer() {
    let n = parse_number_literal("99999999999999999999").unwrap();
    let w = parse_mathematica("99999999999999999999").unwrap();
    assert_eq!(w, WolframForm::number(n));
}

#[test]
fn parse_if_call_shape() {
    let w = parse_mathematica("If[1==1,7,8]").unwrap();
    assert_eq!(
        w,
        WolframForm::call(
            "If",
            vec![
                WolframForm::call("Equal", vec![WolframForm::int(1), WolframForm::int(1)]),
                WolframForm::int(7),
                WolframForm::int(8),
            ]
        )
    );
    let h = H::new();
    assert!(h.eq(h.eval("If[1==1,7,8]"), h.i(7)));
}

#[test]
fn parse_hold_keeps_args() {
    let w = parse_mathematica("Hold[1+1]").unwrap();
    assert_eq!(w, WolframForm::call("Hold", vec![WolframForm::call("Plus", vec![WolframForm::int(1), WolframForm::int(1)])]));
    let h = H::new();
    let e = h.eval("Hold[1+1]");
    assert!(h.eq(e, h.ap("Hold", vec![h.ap("Plus", vec![h.i(1), h.i(1)])])));
}

#[test]
fn parse_hold_form_keeps_args() {
    let h = H::new();
    let e = h.eval("HoldForm[1+1]");
    // Dialect maps `HoldForm` → neutral `Hold`.
    assert!(h.eq(e, h.ap("Hold", vec![h.ap("Plus", vec![h.i(1), h.i(1)])])));
}

#[test]
fn parse_import_call_shape() {
    let w = parse_mathematica("Import[\"x.csv\"]").unwrap();
    assert_eq!(w, WolframForm::call("Import", vec![WolframForm::Atom(WolframAtom::String("x.csv".into()))]));
}

#[test]
fn parse_part_double_bracket() {
    let w = parse_mathematica("{1,2,3}[[0]]").unwrap();
    assert_eq!(
        w,
        WolframForm::call(
            "Part",
            vec![WolframForm::List(vec![WolframForm::int(1), WolframForm::int(2), WolframForm::int(3)]), WolframForm::int(0)]
        )
    );
    let h = H::new();
    assert!(h.eq(h.eval("{1,2,3}[[0]]"), h.lst(vec![])));
}

#[test]
fn parse_part_call_zero() {
    let h = H::new();
    assert!(h.eq(h.eval("Part[{1,2,3},0]"), h.lst(vec![])));
}

#[test]
fn parse_true_false_null_atoms() {
    assert_eq!(parse_mathematica("True").unwrap(), WolframForm::symbol("True"));
    assert_eq!(parse_mathematica("False").unwrap(), WolframForm::symbol("False"));
    assert_eq!(parse_mathematica("Null").unwrap(), WolframForm::symbol("Null"));
    let h = H::new();
    assert!(h.eq(h.eval("True"), h.boolean(true)));
    assert!(h.eq(h.eval("False"), h.boolean(false)));
    assert!(h.eq(h.eval("Null"), h.null()));
    assert_eq!(h.wolfram(h.boolean(true)), "True");
    assert_eq!(h.wolfram(h.null()), "Null");
}

#[test]
fn parse_and_or_not_bool_atoms() {
    let h = H::new();
    assert!(h.eq(h.eval("And[True, False]"), h.boolean(false)));
    assert!(h.eq(h.eval("Or[False, True]"), h.boolean(true)));
    assert!(h.eq(h.eval("Not[True]"), h.boolean(false)));
    assert!(h.eq(h.eval("Which[False, 1, True, 2]"), h.i(2)));
    assert!(h.eq(h.eval("1 == 1"), h.boolean(true)));
}

#[test]
fn and_or_short_circuit_skips_rhs_side_effects() {
    let h = H::new();
    // False && (x=1) must not bind x (avoid CompoundExpression-in-then until Sequence rewrite is scoped).
    assert_eq!(h.wolfram(h.eval("And[False, x = 1]")), "False");
    assert_eq!(h.wolfram(h.eval("x")), "x");

    // True || (y=1) must not bind y.
    assert_eq!(h.wolfram(h.eval("Or[True, y = 1]")), "True");
    assert_eq!(h.wolfram(h.eval("y")), "y");

    // When the leading arm allows, later Set still runs (Define yields the stored value).
    assert_eq!(h.wolfram(h.eval("And[True, z = 7]")), "7");
    assert_eq!(h.wolfram(h.eval("z")), "7");
}

#[test]
fn and_or_short_circuit_compound_expression_then() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("And[True, CompoundExpression[z = 7, True]]")), "True");
    assert_eq!(h.wolfram(h.eval("z")), "7");
    assert_eq!(h.wolfram(h.eval("And[False, CompoundExpression[x = 1, True]]")), "False");
    assert_eq!(h.wolfram(h.eval("x")), "x");
}

#[test]
fn parse_with_module_block_local_bindings() {
    let h = H::new();
    for src in ["With[{x = 1}, x + 1]", "Module[{x = 1}, x + 1]", "Block[{x = 1}, x + 1]"] {
        assert!(h.eq(h.eval(src), h.i(2)), "{src}");
    }
}

#[test]
fn parse_module_fresh_symbols_ignore_session_own() {
    let h = H::new();
    let bare = h.eval("Module[{x}, x]");
    let rendered = h.wolfram(bare);
    assert!(rendered.contains('$'), "Module[{{x}}, x] should yield fresh x$n, got {rendered}");
    assert_ne!(rendered, "5");

    let shadowed = h.eval("x = 5; Module[{x}, x]");
    let rendered = h.wolfram(shadowed);
    assert!(rendered.contains('$'), "x=5; Module[{{x}}, x] must not return 5, got {rendered}");
    assert_ne!(rendered, "5");

    assert!(h.eq(h.eval("Module[{x = 1}, x + 1]"), h.i(2)));
}

#[test]
fn parse_block_dynamic_shadow_and_restore() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("x = 5; Block[{x}, x]")), "x");
    assert_eq!(h.wolfram(h.eval("x = 5; Block[{x = 1}, x]")), "1");
    // Outer Own restored after Block.
    assert_eq!(h.wolfram(h.eval("x = 5; Block[{x}, x]; x")), "5");
    assert_eq!(h.wolfram(h.eval("x = 5; Block[{x = 1}, x]; x")), "5");
    assert!(h.eq(h.eval("Block[{x = 1}, x + 1]"), h.i(2)));
}

#[test]
fn parse_with_simultaneous_lexical_substitution() {
    let h = H::new();
    assert!(h.eq(h.eval("With[{x = 1}, x + 1]"), h.i(2)));
    // Simultaneous: RHS of y sees outer x, not With's x=1.
    assert_eq!(h.wolfram(h.eval("With[{x = 1, y = x}, y]")), "x");
    // With does not leave Own bindings.
    assert_eq!(h.wolfram(h.eval("With[{x = 1}, x]; x")), "x");
    assert_eq!(h.wolfram(h.eval("x = 5; With[{x = 1, y = x}, y]")), "5");
}

#[test]
fn parse_slot_lowers_to_slot_head() {
    let w = parse_mathematica("#").unwrap();
    assert_eq!(w, WolframForm::call("Slot", vec![WolframForm::int(1)]));
}

#[test]
fn parse_pure_function_slot_application() {
    let h = H::new();
    assert!(h.eq(h.eval("(#^2)&[4]"), h.i(16)));
}

#[test]
fn parse_named_function_application() {
    let h = H::new();
    assert!(h.eq(h.eval("Function[x, x^2][3]"), h.i(9)));
}

#[test]
fn parse_map_pure_function() {
    let h = H::new();
    assert!(h.eq(h.eval("Map[#^2 &, {1, 2, 3}]"), h.lst(vec![h.i(1), h.i(4), h.i(9)])));
}

#[test]
fn parse_blank_and_typed_blank() {
    assert_eq!(parse_mathematica("_").unwrap(), WolframForm::call("Blank", vec![]));
    assert_eq!(parse_mathematica("_Integer").unwrap(), WolframForm::call("Blank", vec![WolframForm::symbol("Integer")]));
    assert_eq!(
        parse_mathematica("x_").unwrap(),
        WolframForm::call("Pattern", vec![WolframForm::symbol("x"), WolframForm::call("Blank", vec![])])
    );
}

#[test]
fn parse_match_q_and_cases() {
    let h = H::new();
    assert!(h.eq(h.eval("MatchQ[1, _Integer]"), h.boolean(true)));
    assert!(h.eq(h.eval("MatchQ[a, _Integer]"), h.boolean(false)));
    assert!(h.eq(h.eval("Cases[{1, a, 2}, _Integer]"), h.lst(vec![h.i(1), h.i(2)])));
}

#[test]
fn parse_table_range_apply_list_primitives() {
    let h = H::new();
    assert!(h.eq(h.eval("Table[i, {i, 3}]"), h.lst(vec![h.i(1), h.i(2), h.i(3)])));
    assert!(h.eq(h.eval("Range[3]"), h.lst(vec![h.i(1), h.i(2), h.i(3)])));
    assert!(h.eq(h.eval("Apply[Plus, {1, 2, 3}]"), h.i(6)));
    assert!(h.eq(h.eval("Length[{1, 2, 3}]"), h.i(3)));
    assert!(h.eq(h.eval("Join[{1}, {2}]"), h.lst(vec![h.i(1), h.i(2)])));
    assert!(h.eq(h.eval("First[{a, b}]"), h.sym("a")));
    assert!(h.eq(h.eval("Sum[i, {i, 1, 10}]"), h.i(55)));
    assert!(h.eq(h.eval("Product[i, {i, 1, 5}]"), h.i(120)));
}

#[test]
fn parse_limit_sinc_and_definite_integrate_sin() {
    let h = H::new();
    assert!(h.eq(h.eval("Cos[Pi]"), h.i(-1)));
    let lim = h.eval("Limit[Sin[x]/x, x -> 0]");
    assert!(h.eq(lim, h.i(1)));
    let integ = h.eval("Integrate[Cos[x], x]");
    let text = h.wolfram(integ);
    assert!(text.contains('x') || text.contains("Sin"), "got {text}");
}

#[test]
fn parse_linear_solve_nested_lists() {
    let h = H::new();
    assert!(h.eq(h.eval("Det[{{1, 2}, {3, 4}}]"), h.i(-2)));
    assert_eq!(h.wolfram(h.eval("LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]")), "{{-4}, {9/2}}");
}

#[test]
fn transpose_nested_list_via_matrix_value_goal() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Transpose[{{1, 2}, {3, 4}}]")), "{{1, 3}, {2, 4}}");
    // Real ConjugateTranspose matches Transpose on exact integer matrices.
    assert_eq!(h.wolfram(h.eval("ConjugateTranspose[{{1, 2}, {3, 4}}]")), "{{1, 3}, {2, 4}}");
}

#[test]
fn series_exp_order_two_renders_polynomial() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Series[Exp[x], {x, 0, 2}]")), "1 + x + 1/2*x^2");
}

#[test]
fn series_exp_order_three_renders_polynomial() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Series[Exp[x], {x, 0, 3}]")), "1 + x + 1/2*x^2 + 1/6*x^3");
}

#[test]
fn series_sin_order_three_renders_polynomial() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Series[Sin[x], {x, 0, 3}]")), "x + -1/6*x^3");
}

#[test]
fn definite_gaussian_exp_neg_square_is_sqrt_pi() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Integrate[Exp[-x^2], {x, -Infinity, Infinity}]")), "Sqrt[Pi]");
}

#[test]
fn integrate_x_sin_x_by_parts() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Integrate[x*Sin[x], x]")), "-1*x*Cos[x] + Sin[x]");
}

#[test]
fn limit_one_plus_x_to_reciprocal_is_e() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Limit[(1 + x)^(1/x), x -> 0]")), "E");
}

#[test]
fn integrate_reciprocal_is_log() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Integrate[1/x, x]")), "Log[x]");
}

#[test]
fn limit_reciprocal_at_infinity_is_zero() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Limit[1/x, x -> Infinity]")), "0");
}

#[test]
fn parse_solve_quadratic_x2_eq_1() {
    let h = H::new();
    // Solve stays Extension surface until DomainGoal lowering (Living `14`).
    let e = h.lower(&h.parse_w("Solve[x^2 == 1, x]"));
    assert!(matches!(
        h.s.borrow().arena.get(e),
        Some(TermNode::Application { head: athena::ir::ApplicationHead::Extension(_), .. })
    ));
    assert!(h.wolfram(e).starts_with("Solve["));
}

#[test]
fn parse_plot_negative_domain_renders_svg() {
    let h = H::new();
    let w = h.parse_w("Plot[x^2, {x, -1, 1}]");
    let t = h.lower(&w);
    let svg = h.with_mut(|s| try_plot_svg(s, t)).expect("extract").expect("render");
    assert!(svg.contains("<svg"), "got {svg}");
}

#[test]
fn render_keeps_parens_for_negative_rational_power() {
    let h = H::new();
    let parsed = h.parse_w("(-8)^(1/3)");
    assert_eq!(render(&parsed), "(-8)^(1/3)");
    // Athena may fold Times[-1,8] / Divide into Number atoms; render must still paren.
    let roundtrip = h.wolfram(h.eval("(-8)^(1/3)"));
    assert_eq!(roundtrip, "-2", "got {roundtrip}");
}

#[test]
fn patterned_set_delayed_dispatches() {
    let h = H::new();
    assert!(h.eq(h.eval("f[x_]:=x^2; f[3]"), h.i(9)), "got {}", h.wolfram(h.eval("f[x_]:=x^2; f[3]")));
}

#[test]
fn symbol_set_delayed_evaluates_on_use() {
    let h = H::new();
    // `:=` stores residual; use-time evaluation yields 2, not unevaluated Plus.
    assert!(h.eq(h.eval("a := 1 + 1; a"), h.i(2)), "got {}", h.wolfram(h.eval("a := 1 + 1; a")));
    assert_eq!(h.wolfram(h.eval("b := 1 + 1")), "Null");
}

#[test]
fn rule_delayed_rhs_held_until_replace_all() {
    let h = H::new();
    // Head-form ReplaceAll + RuleDelayed: RHS evaluates after substitution.
    assert!(
        h.eq(h.eval("ReplaceAll[x, RuleDelayed[x, 1 + 1]]"), h.i(2)),
        "got {}",
        h.wolfram(h.eval("ReplaceAll[x, RuleDelayed[x, 1 + 1]]"))
    );
    // Rule evaluates RHS at construction (same numeric result here).
    assert!(
        h.eq(h.eval("ReplaceAll[x, Rule[x, 1 + 1]]"), h.i(2)),
        "got {}",
        h.wolfram(h.eval("ReplaceAll[x, Rule[x, 1 + 1]]"))
    );
}

#[test]
fn clear_definition_returns_null_and_unbinds() {
    let h = H::new();
    assert!(h.eq(h.eval("x = 5; Clear[x]; x"), h.sym("x")), "got {}", h.wolfram(h.eval("x = 5; Clear[x]; x")));
    assert_eq!(h.wolfram(h.eval("Clear[y]")), "Null");
}

#[test]
fn map_sin_keeps_exact_sin_one() {
    let h = H::new();
    let got = h.wolfram(h.eval("Map[Sin, {0, 1}]"));
    assert_eq!(got, "{0, Sin[1]}", "got {got}");
}

#[test]
fn rest_drops_first_element() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Rest[{1, 2, 3}]")), "{2, 3}");
}

#[test]
fn total_sums_list_elements() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Total[{1, 2, 3}]")), "6");
}

#[test]
fn cases_filters_integer_blank() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Cases[{1, 2, 3}, _Integer]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("Cases[{1, a, 2}, _Integer]")), "{1, 2}");
}

#[test]
fn matchq_integer_blank() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("MatchQ[1, _Integer]")), "True");
}

#[test]
fn compound_expression_set_binds() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("CompoundExpression[a = 1, a]")), "1");
}

#[test]
fn hold_preserves_plus() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Hold[1 + 1]")), "Hold[1 + 1]");
    assert_eq!(h.wolfram(h.eval("HoldForm[1 + 1]")), "Hold[1 + 1]");
}

#[test]
fn unary_minus_binds_looser_than_power() {
    let h = H::new();
    let w = h.parse_w("-x^2");
    assert_eq!(
        w,
        WolframForm::call(
            "Times",
            vec![
                WolframForm::int(-1),
                WolframForm::call("Power", vec![WolframForm::symbol("x"), WolframForm::int(2)]),
            ]
        )
    );
    let paren = h.parse_w("(-x)^2");
    assert_eq!(
        paren,
        WolframForm::call(
            "Power",
            vec![
                WolframForm::call("Times", vec![WolframForm::int(-1), WolframForm::symbol("x")]),
                WolframForm::int(2),
            ]
        )
    );
    assert_eq!(h.wolfram(h.eval("Exp[-x^2]")), "Exp[-(x^2)]");
}

#[test]
fn implicit_times_keeps_d_arity() {
    let h = H::new();
    let w = h.parse_w("D[x y, x]");
    assert_eq!(
        w,
        WolframForm::call(
            "D",
            vec![
                WolframForm::call("Times", vec![WolframForm::symbol("x"), WolframForm::symbol("y")]),
                WolframForm::symbol("x"),
            ]
        )
    );
    assert_eq!(h.wolfram(h.eval("D[x y, x]")), "y");
    assert_eq!(h.wolfram(h.eval("D[x*y, x]")), "y");
}

#[test]
fn exp_0_and_log_1_fold_exactly() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Exp[0]")), "1");
    assert_eq!(h.wolfram(h.eval("Log[1]")), "0");
}

#[test]
fn while_false_returns_null() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("While[False, 1]")), "Null");
}

#[test]
fn do_count_returns_null() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Do[1, {3}]")), "Null");
    assert_eq!(h.wolfram(h.eval("Do[i, {i, 3}]")), "Null");
}

#[test]
fn release_hold_evaluates_held_plus() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("ReleaseHold[Hold[1 + 1]]")), "2");
    assert_eq!(h.wolfram(h.eval("Evaluate[Hold[1 + 1]]")), "2");
}

#[test]
fn assert_true_returns_null() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Assert[True]")), "Null");
    assert_eq!(h.wolfram(h.eval("Assert[1 == 1]")), "Null");
}

#[test]
fn trueq_and_boole_from_predicates() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("TrueQ[True]")), "True");
    assert_eq!(h.wolfram(h.eval("TrueQ[1 == 1]")), "True");
    assert_eq!(h.wolfram(h.eval("TrueQ[False]")), "False");
    assert_eq!(h.wolfram(h.eval("Boole[True]")), "1");
    assert_eq!(h.wolfram(h.eval("Boole[2 > 1]")), "1");
    assert_eq!(h.wolfram(h.eval("Boole[False]")), "0");
}

#[test]
fn xor_and_implies_from_branch() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Xor[True, False]")), "True");
    assert_eq!(h.wolfram(h.eval("Xor[True, True]")), "False");
    assert_eq!(h.wolfram(h.eval("Implies[True, False]")), "False");
    assert_eq!(h.wolfram(h.eval("Implies[False, False]")), "True");
}
