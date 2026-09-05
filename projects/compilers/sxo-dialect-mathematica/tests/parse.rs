//! Integration tests for Mathematica parse (session arena `TermId`).

use std::cell::RefCell;

use athena::{
    AthenaEngine,
    ir::Atom,
    ir::TermNode,
    numeric::number_from_wire,
    runtime::values::arena::push_application_named,
    runtime::values::arena::push_bool,
    runtime::values::arena::push_int,
    runtime::values::arena::push_list,
    runtime::values::arena::push_null,
    runtime::values::arena::push_symbol_name,
    Session,
    types::TermId,
};
use athena_types::WireNumber;
use sxo_dialect_mathematica::{
    WAtom, WExpr, lower_request, lower_wexpr, parse_mathematica, parse_number_literal, render, try_plot_svg,
    wexpr_from_session,
};

type Tid = TermId;

struct H {
    s: RefCell<Session>,
}

impl H {
    fn new() -> Self {
        Self { s: RefCell::new(Session::new()) }
    }

    fn parse_w(&self, input: &str) -> WExpr {
        parse_mathematica(input).unwrap()
    }

    fn lower(&self, w: &WExpr) -> Tid {
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
        push_application_named(&mut self.s.borrow_mut(), head, args)
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

    fn rational(&self, n: i64, d: i64) -> Tid {
        let wire = WireNumber::rational_i64(n, d).unwrap();
        let num = number_from_wire(&wire).unwrap();
        let span = athena::types::SourceSpan::default();
        self.s.borrow_mut().arena.push(TermNode::Atom(Atom::Number(num)), span)
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
    assert_eq!(e, WExpr::call("Sin", vec![WExpr::symbol("x")]));
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
    assert_eq!(w, WExpr::call("CompoundExpression", vec![WExpr::symbol("a"), WExpr::symbol("b")]));
}

#[test]
fn parse_root_semicolon_returns_last() {
    let h = H::new();
    assert!(h.eq(h.eval("1 + 2; 3 * 4"), h.i(12)));
}

#[test]
fn parse_equal_and_factorial() {
    let w = parse_mathematica("2 == 2").unwrap();
    assert_eq!(w, WExpr::call("Equal", vec![WExpr::int(2), WExpr::int(2)]));
    let h = H::new();
    assert!(h.eq(h.eval("2 == 2"), h.boolean(true)));
    assert!(h.eq(h.eval("5!"), h.i(120)));
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
    let h = H::new();
    assert!(h.eq(h.eval("If[1==1,7,8]"), h.i(7)));
}

#[test]
fn parse_hold_keeps_args() {
    let w = parse_mathematica("Hold[1+1]").unwrap();
    assert_eq!(w, WExpr::call("Hold", vec![WExpr::call("Plus", vec![WExpr::int(1), WExpr::int(1)])]));
    let h = H::new();
    let e = h.eval("Hold[1+1]");
    assert!(h.eq(e, h.ap("Hold", vec![h.ap("Plus", vec![h.i(1), h.i(1)])])));
}

#[test]
fn parse_hold_form_keeps_args() {
    let h = H::new();
    let e = h.eval("HoldForm[1+1]");
    assert!(h.eq(e, h.ap("HoldForm", vec![h.ap("Plus", vec![h.i(1), h.i(1)])])));
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
    let h = H::new();
    assert!(h.eq(h.eval("{1,2,3}[[0]]"), h.sym("List")));
}

#[test]
fn parse_part_call_zero() {
    let h = H::new();
    assert!(h.eq(h.eval("Part[{1,2,3},0]"), h.sym("List")));
}

#[test]
fn parse_true_false_null_atoms() {
    assert_eq!(parse_mathematica("True").unwrap(), WExpr::symbol("True"));
    assert_eq!(parse_mathematica("False").unwrap(), WExpr::symbol("False"));
    assert_eq!(parse_mathematica("Null").unwrap(), WExpr::symbol("Null"));
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
fn parse_with_module_block_local_bindings() {
    let h = H::new();
    for src in ["With[{x = 1}, x + 1]", "Module[{x = 1}, x + 1]", "Block[{x = 1}, x + 1]"] {
        assert!(h.eq(h.eval(src), h.i(2)), "{src}");
    }
}

#[test]
fn parse_slot_lowers_to_slot_head() {
    let w = parse_mathematica("#").unwrap();
    assert_eq!(w, WExpr::call("Slot", vec![WExpr::int(1)]));
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
    assert!(h.eq(h.eval("Limit[Sin[x]/x, x -> 0]"), h.i(1)));
    assert!(h.eq(h.eval("Integrate[Sin[x], {x, 0, Pi}]"), h.i(2)));
    assert!(h.eq(h.eval("Cos[Pi]"), h.i(-1)));
}

#[test]
fn parse_linear_solve_nested_lists() {
    let h = H::new();
    assert!(h.eq(
        h.eval("LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]"),
        h.lst(vec![h.lst(vec![h.i(-4)]), h.lst(vec![h.rational(9, 2)])])
    ));
    assert!(h.eq(h.eval("Det[{{1, 2}, {3, 4}}]"), h.i(-2)));
}

#[test]
fn parse_solve_quadratic_x2_eq_1() {
    let h = H::new();
    let e = h.eval("Solve[x^2 == 1, x]");
    assert!(h.eq(
        e,
        h.lst(vec![
            h.lst(vec![h.ap("Rule", vec![h.sym("x"), h.i(-1)])]),
            h.lst(vec![h.ap("Rule", vec![h.sym("x"), h.i(1)])]),
        ])
    ));
    assert_eq!(h.wolfram(e), "{{x -> -1}, {x -> 1}}");
}

#[test]
fn parse_plot_negative_domain_renders_svg() {
    let h = H::new();
    let w = h.parse_w("Plot[x^2, {x, -1, 1}]");
    let t = h.lower(&w);
    let svg = h.with_mut(|s| try_plot_svg(s, t)).expect("extract").expect("render");
    assert!(svg.contains("<svg"), "got {svg}");
}
