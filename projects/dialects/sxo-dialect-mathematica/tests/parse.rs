//! Integration tests for Mathematica parse (session arena `TermId`).

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session,
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
            Ok(result_id) => {
                let result = s.results.get(result_id).unwrap_or_else(|| panic!("missing ResultId after eval: {input}"));
                result.symbolic_term.unwrap_or_else(|| {
                    panic!(
                        "result has no symbolic_term after eval: {input} (status={}, coverage={})",
                        result.status.name(),
                        result.coverage.name()
                    )
                })
            }
            Err(err) => panic!("execute_request failed for {input}: {err}"),
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
fn parse_slot_n_keeps_index() {
    let w = parse_mathematica("#2").unwrap();
    assert_eq!(w, WolframForm::call("Slot", vec![WolframForm::int(2)]));
    assert_eq!(render(&w), "#2");

    let pure = parse_mathematica("#2 &").unwrap();
    // Must not become Times[Slot[1], 2] / Function[$slot1, $slot1*2].
    let text = format!("{pure:?}");
    assert!(text.contains("Slot"), "got {pure:?}");
    assert!(!matches!(pure.head_name(), Some("Times")));

    let mapped = parse_mathematica("MapIndexed[#2 &, {a, b}]").unwrap();
    assert_eq!(mapped.head_name(), Some("MapIndexed"));
    let rendered = render(&mapped);
    assert!(rendered.contains("#2") || rendered.contains("Slot[2]"), "got {rendered}");
    assert!(!rendered.contains("$slot1*2"));
    assert!(!rendered.contains("$slot1"));
}

#[test]
fn parse_message_name_and_information_prefix() {
    let msg = parse_mathematica("f::x").unwrap();
    assert_eq!(msg, WolframForm::call("MessageName", vec![WolframForm::symbol("f"), WolframForm::symbol("x")]));
    assert_eq!(render(&msg), "f::x");

    let wrapped = parse_mathematica("Message[f::x]").unwrap();
    assert_eq!(wrapped.head_name(), Some("Message"));
    assert_eq!(render(&wrapped), "Message[f::x]");

    let info = parse_mathematica("??Plus").unwrap();
    assert_eq!(info.head_name(), Some("Information"));
    assert_eq!(render(&info), "Information[Plus]");

    let h = H::new();
    // Must not silently collapse MessageName / Information to bare symbols.
    assert_ne!(h.wolfram(h.eval("Message[f::x]")), "Message[f, x]");
    let got_info = h.wolfram(h.eval("??Plus"));
    assert!(got_info.contains("Information") || got_info.contains("Plus"), "got {got_info}");
    assert_ne!(got_info, "Plus");
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
fn set_nested_list_binds_matrix_domain_object() {
    let h = H::new();
    let w = h.parse_w("A={{1, 2}, {3, 4}}");
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &w);
    assert!(matches!(request, athena::api::AthenaRequest::Command(athena::api::SessionCommand::DefineMatrix { .. })));
    AthenaEngine::new().execute_request(&mut s, request).expect("define matrix");
    let symbol = s.arena.symbols_mut().intern("A");
    assert!(s.matrix_binding(symbol).is_some());
    assert!(s.defs.binding(symbol).is_none());
}

#[test]
fn transpose_symbol_after_set_uses_matrix_binding() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Transpose[A]")), "{{1, 3}, {2, 4}}");
}

#[test]
fn unary_matrix_goals_resolve_symbol_bindings() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; MatrixRank[A]")), "2");
    assert_eq!(h.wolfram(h.eval("B={{1, 2}, {3, 4}}; Tr[B]")), "5");
    assert_eq!(h.wolfram(h.eval("C={{1, 2}, {3, 4}}; Inverse[C]")), "{{-2, 1}, {3/2, -1/2}}");
    // Living 16: singular Inverse projects Inverse[Singular] residual (not Diagnostic hard fail).
    let singular = h.wolfram(h.eval("Inverse[{{1, 2}, {2, 4}}]"));
    assert!(
        singular.contains("Inverse") && singular.contains("Singular"),
        "expected Inverse Singular residual, got {singular}"
    );
    assert_eq!(h.wolfram(h.eval("D={{1, 2}, {3, 4}}; Det[D]")), "-2");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; First[A]")), "{1, 2}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Rest[A]")), "{3, 4}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Flatten[A]")), "{1, 2, 3, 4}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Most[A]")), "{1, 2}");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 3}; Reverse[V]")), "{3, 2, 1}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}, {5, 6}}; Take[A, 2]")), "{{1, 2}, {3, 4}}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}, {5, 6}}; Drop[A, 1]")), "{{3, 4}, {5, 6}}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; B={{5, 6}}; Join[A, B]")), "{{1, 2}, {3, 4}, {5, 6}}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Append[A, {5, 6}]")), "{{1, 2}, {3, 4}, {5, 6}}");
    assert_eq!(h.wolfram(h.eval("V={1, 2}; Append[V, 3]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("V={2, 3}; Prepend[V, 0]")), "{0, 2, 3}");
    assert_eq!(h.wolfram(h.eval("V={3, 1, 2}; Sort[V]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 3, 4}; Partition[V, 2]")), "{{1, 2}, {3, 4}}");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 1, 3, 2}; DeleteDuplicates[V]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 1, 3}; MemberQ[V, 2]")), "True");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 1, 3}; Count[V, 1]")), "2");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 1}; FreeQ[V, 3]")), "True");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 1}; Position[V, 1]")), "{{1}, {3}}");
    assert_eq!(h.wolfram(h.eval("V={1, 2}; PadLeft[V, 4]")), "{0, 0, 1, 2}");
    assert_eq!(h.wolfram(h.eval("A={1, 2}; B={9, 8}; Riffle[A, B]")), "{1, 9, 2, 8}");
    assert_eq!(h.wolfram(h.eval("A={3, 1, 2}; B={2, 4, 1}; Union[A, B]")), "{1, 2, 3, 4}");
    assert_eq!(h.wolfram(h.eval("A={3, 1, 2}; B={2, 4, 1}; Intersection[A, B]")), "{1, 2}");
    assert_eq!(h.wolfram(h.eval("V={10, 20, 30}; Extract[V, 2]")), "20");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Extract[A, 2]")), "{3, 4}");
    assert_eq!(h.wolfram(h.eval("V=ConstantArray[7, 3]; Length[V]")), "3");
    assert_eq!(h.wolfram(h.eval("V=ConstantArray[7, 3]; MemberQ[V, 7]")), "True");
    assert_eq!(h.wolfram(h.eval("V=Range[4]; Length[V]")), "4");
    assert_eq!(h.wolfram(h.eval("V=Range[2, 6, 2]; MemberQ[V, 4]")), "True");
    assert_eq!(h.wolfram(h.eval("A=ConstantArray[5, {2, 3}]; Dimensions[A]")), "{2, 3}");
    assert_eq!(h.wolfram(h.eval("A=ConstantArray[5, {2, 3}]; First[A]")), "{5, 5, 5}");
    assert_eq!(h.wolfram(h.eval("V=Range[3]; DiagonalMatrix[V]")), "{{1, 0, 0}, {0, 2, 0}, {0, 0, 3}}");
    assert_eq!(h.wolfram(h.eval("A=IdentityMatrix[2]; Det[A]")), "1");
    assert_eq!(h.wolfram(h.eval("Det[ConstantArray[1, {2, 2}]]")), "0");
    assert_eq!(h.wolfram(h.eval("Total[ConstantArray[1, {2, 2}]]")), "{2, 2}");
    assert_eq!(h.wolfram(h.eval("Accumulate[Range[3]]")), "{1, 3, 6}");
    assert_eq!(h.wolfram(h.eval("Differences[Range[4]]")), "{1, 1, 1}");
    assert_eq!(h.wolfram(h.eval("Sort[{3, 1, 2}]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("PadLeft[{1, 2}, 4]")), "{0, 0, 1, 2}");
    assert_eq!(h.wolfram(h.eval("Join[{1, 2}, {3}]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("Union[{3, 1}, {2, 1}]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("Riffle[{1, 2}, {9, 8}]")), "{1, 9, 2, 8}");
    assert_eq!(h.wolfram(h.eval("DiagonalMatrix[{1, 2}]")), "{{1, 0}, {0, 2}}");
    assert_eq!(h.wolfram(h.eval("A=IdentityMatrix[2]; Dimensions[A]")), "{2, 2}");
    assert_eq!(h.wolfram(h.eval("A=IdentityMatrix[3]; Det[A]")), "1");
    // Living 16: symbolic diagonal stays residual (no nested-list rebuild).
    assert_eq!(h.wolfram(h.eval("DiagonalMatrix[{x}]")), "DiagonalMatrix[{x}]");
}

#[test]
fn part_on_matrix_binding() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; A[[1, 2]]")), "2");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; A[[2]]")), "{3, 4}");
    assert_eq!(h.wolfram(h.eval("V={10, 20, 30}; V[[2]]")), "20");
}

#[test]
fn part_store_on_matrix_binding() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; A[[1, 2]] = 9; A[[1, 2]]")), "9");
    assert_eq!(h.wolfram(h.eval("V={10, 20, 30}; V[[2]] = 8; V")), "{10, 8, 30}");
    // Living 16: in-place StoreIndex keeps Own usable for follow-on matrix ops.
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; A[[1, 1]] = 0; Det[A]")), "-6");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 3}; V[[3]] = 9; Length[V]")), "3");
}

#[test]
fn nested_form_transpose_feeds_inverse_matrix_operand() {
    let h = H::new();
    // Living 16: Form wrappers unwrap at lowering — no Term Collection reverse recognition.
    assert_eq!(h.wolfram(h.eval("Inverse[Transpose[{{1, 2}, {3, 4}}]]")), "{{-2, 3/2}, {1, -1/2}}");
    let w = h.parse_w("Inverse[Transpose[{{1, 2}, {3, 4}}]]");
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &w);
    assert!(matches!(
        request,
        athena::api::AthenaRequest::Goal(athena::api::DomainGoal::Dispatch(athena::domains::DomainRequest::LinearAlgebra(
            athena::domains::linear_algebra::LinearAlgebraRequest::Inverse { .. }
        )))
    ));
}

#[test]
fn times_form_matrices_lower_to_hadamard_goal() {
    let h = H::new();
    let w = h.parse_w("{{1, 2}, {3, 4}}*{{5, 6}, {7, 8}}");
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &w);
    assert!(matches!(
        request,
        athena::api::AthenaRequest::Goal(athena::api::DomainGoal::Dispatch(athena::domains::DomainRequest::LinearAlgebra(
            athena::domains::linear_algebra::LinearAlgebraRequest::Hadamard { .. }
        )))
    ));
    drop(s);
    // Living 16: Mathematica `Times` is Hadamard, not MatMul.
    assert_eq!(h.wolfram(h.eval("{{1, 2}, {3, 4}}*{{5, 6}, {7, 8}}")), "{{5, 12}, {21, 32}}");
}

#[test]
fn det_goal_uses_matrix_operand_binding() {
    let h = H::new();
    let w = h.parse_w("A={{1, 2}, {3, 4}}; Det[A]");
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &w);
    // CompoundExpression: last step must be LinearAlgebra Det goal (not Semantic Determinant Term).
    match &request {
        athena::api::AthenaRequest::Control(athena::api::ControlPlan::Sequence { steps }) => {
            let last = steps.last().expect("steps");
            assert!(matches!(
                last,
                athena::api::AthenaRequest::Goal(athena::api::DomainGoal::Dispatch(
                    athena::domains::DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::Det { .. }
                    )
                ))
            ));
        }
        other => panic!("expected Sequence, got {other:?}"),
    }
}

#[test]
fn matrix_times_resolves_symbol_bindings() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; B={{5, 6}, {7, 8}}; Dot[A, B]")), "{{19, 22}, {43, 50}}");
    // Living 16: Mathematica `Times` on matrices is Hadamard. `Dot` remains MatMul.
    assert_eq!(h.wolfram(h.eval("P={{1, 2}, {3, 4}}; Q={{5, 6}, {7, 8}}; P*Q")), "{{5, 12}, {21, 32}}");
}

#[test]
fn binary_matrix_goals_resolve_symbol_bindings() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; B={{5}, {6}}; LinearSolve[A, B]")), "{{-4}, {9/2}}");
    // Living 16: ExactSolve particular publishes MatrixResult via typed MatrixRef operands.
    assert_eq!(h.wolfram(h.eval("A=IdentityMatrix[2]; B={{3}, {5}}; LinearSolve[A, B]")), "{{3}, {5}}");
    assert_eq!(h.wolfram(h.eval("LinearSolve[{{1, 2}, {3, 4}}, {{5}, {11}}]")), "{{1}, {2}}");
    // Living 16: inconsistent ExactSolve projects empty list; Infinite keeps a particular.
    assert_eq!(h.wolfram(h.eval("LinearSolve[{{1, 2}, {2, 4}}, {{1}, {0}}]")), "{}");
    assert_eq!(h.wolfram(h.eval("LinearSolve[{{1, 2}, {2, 4}}, {{2}, {4}}]")), "{{2}, {0}}");
    // Living 16: machine-float Form → MachineSolve Singular residual (not exact Infinite).
    let singular = h.wolfram(h.eval("LinearSolve[{{1.0, 2.0}, {2.0, 4.0}}, {{1.0}, {0.0}}]"));
    assert!(singular.contains("LinearSolve") && singular.contains("Singular"), "expected Singular residual, got {singular}");
    assert_eq!(h.wolfram(h.eval("M={{1, 2}, {3, 4}}; V={{1}, {1}}; Dot[M, V]")), "{3, 7}");
    assert_eq!(h.wolfram(h.eval("U={1, 0, 0}; W={0, 1, 0}; Cross[U, W]")), "{0, 0, 1}");
}

#[test]
fn matrix_rank_rank1() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("MatrixRank[{{1, 2}, {2, 4}}]")), "1");
}

#[test]
fn identity_matrix_two() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("IdentityMatrix[2]")), "{{1, 0}, {0, 1}}");
}

#[test]
fn dimensions_of_nested_list() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Dimensions[{{1, 2}, {3, 4}}]")), "{2, 2}");
}

#[test]
fn dimensions_of_matrix_binding() {
    let h = H::new();
    // Living 16: bound MatrixRef shape without nested-list reverse recognition.
    assert_eq!(h.wolfram(h.eval("A={{1, 2, 3}, {4, 5, 6}}; Dimensions[A]")), "{2, 3}");
}

#[test]
fn length_of_matrix_binding() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2, 3}, {4, 5, 6}}; Length[A]")), "2");
    assert_eq!(h.wolfram(h.eval("V={1, 2, 3}; Length[V]")), "3");
}

#[test]
fn diagonal_matrix_from_vector() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("DiagonalMatrix[{1, 2}]")), "{{1, 0}, {0, 2}}");
}

#[test]
fn tr_of_2x2() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Tr[{{1, 2}, {3, 4}}]")), "5");
}

#[test]
fn dot_matrix_vector() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Dot[{{1, 2}, {3, 4}}, {1, 1}]")), "{3, 7}");
}

#[test]
fn cross_ijk() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Cross[{1, 0, 0}, {0, 1, 0}]")), "{0, 0, 1}");
}

#[test]
fn nullspace_rank1() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("NullSpace[{{1, 2}, {2, 4}}]")), "{{-2, 1}}");
    // Living 16: bound MatrixRef / full-rank Identity → typed NullSpace MatrixResult.
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {2, 4}}; NullSpace[A]")), "{{-2, 1}}");
    assert_eq!(h.wolfram(h.eval("A=IdentityMatrix[2]; NullSpace[A]")), "{}");
    // Living 16: IdentityMatrix Form constructor lowers to MatrixOperand without Set.
    assert_eq!(h.wolfram(h.eval("NullSpace[IdentityMatrix[2]]")), "{}");
}

#[test]
fn tril_triu_and_kronecker_product() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("LowerTriangularize[{{1, 2}, {3, 4}}]")), "{{1, 0}, {3, 4}}");
    assert_eq!(h.wolfram(h.eval("UpperTriangularize[{{1, 2}, {3, 4}}]")), "{{1, 2}, {0, 4}}");
    assert_eq!(h.wolfram(h.eval("KroneckerProduct[{1, 2}, {3, 4}]")), "{3, 4, 6, 8}");
}

#[test]
fn norm_34() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Norm[{3, 4}]")), "5");
}

#[test]
fn row_reduce_to_identity() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("RowReduce[{{1, 2}, {3, 4}}]")), "{{1, 0}, {0, 1}}");
    // Living 16: bound MatrixRef → Rref publishes MatrixResult envelope.
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; RowReduce[A]")), "{{1, 0}, {0, 1}}");
    assert_eq!(h.wolfram(h.eval("A=IdentityMatrix[2]; RowReduce[A]")), "{{1, 0}, {0, 1}}");
}

#[test]
fn inverse_identity_matrix() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Inverse[{{1, 0}, {0, 1}}]")), "{{1, 0}, {0, 1}}");
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
fn residue_shifted_simple_pole_is_one() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Residue[1/(z - 1), {z, 1}]")), "1");
}

#[test]
fn residue_exp_over_z_at_zero_is_one() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Residue[Exp[z]/z, {z, 0}]")), "1");
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
fn grad_xy_product() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Grad[x*y, {x, y}]")), "{y, x}");
}

#[test]
fn div_identity_field() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Div[{x, y}, {x, y}]")), "2");
}

#[test]
fn curl_2d_rotation_field() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Curl[{-y, x}, {x, y}]")), "2");
}

#[test]
fn laplace_exp_neg_a_t() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("LaplaceTransform[Exp[-a*t], t, s]")), "(s + a)^(-1)");
}

#[test]
fn fourier_exp_neg_x_squared() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("FourierTransform[Exp[-x^2], x, k]")), "Sqrt[Pi]*Exp[-1/4*k^2]");
}

#[test]
fn z_transform_of_n() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("ZTransform[n, n, z]")), "z*(-1 + z)^(-2)");
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
fn solve_x_squared_eq_one() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Solve[x^2 == 1, x]")), "{{x -> -1}, {x -> 1}}");
}

#[test]
fn solve_linear_two_by_two_rules() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Solve[{x + y == 3, x - y == 1}, {x, y}]")), "{{x -> 2, y -> 1}}");
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
fn map_indexed_second_slot_returns_indices() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("MapIndexed[#2 &, {a, b}]")), "{{1}, {2}}");
    // Must not rewrite #2 into Times[Slot[1], 2].
    assert_ne!(h.wolfram(h.eval("MapIndexed[#2 &, {a, b}]")), "{2*a, 4*b}");
}

#[test]
fn map_thread_applies_head_to_columns() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("MapThread[f, {{1, 2}, {3, 4}}]")), "{f[1, 3], f[2, 4]}");
    assert_eq!(h.wolfram(h.eval("MapThread[Plus, {{1, 2}, {3, 4}}]")), "{4, 6}");
}

#[test]
fn rest_drops_first_element() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Rest[{1, 2, 3}]")), "{2, 3}");
}

#[test]
fn most_and_reverse_list_structure() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Most[{1, 2, 3}]")), "{1, 2}");
    assert_eq!(h.wolfram(h.eval("Reverse[{1, 2, 3}]")), "{3, 2, 1}");
}

#[test]
fn take_and_drop_prefix_slices() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Take[{1, 2, 3, 4}, 2]")), "{1, 2}");
    assert_eq!(h.wolfram(h.eval("Drop[{1, 2, 3, 4}, 2]")), "{3, 4}");
}

#[test]
fn flatten_nested_lists() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Flatten[{{1, 2}, {3}}]")), "{1, 2, 3}");
}

#[test]
fn append_and_prepend_list_elements() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Append[{1, 2}, 3]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("Prepend[{2, 3}, 1]")), "{1, 2, 3}");
}

#[test]
fn member_q_sort_and_delete_duplicates() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("MemberQ[{1, 2, 3}, 2]")), "True");
    assert_eq!(h.wolfram(h.eval("Sort[{3, 1, 2}]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("DeleteDuplicates[{1, 1, 2}]")), "{1, 2}");
}

#[test]
fn count_partition_and_constant_array() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Count[{1, 1, 2}, 1]")), "2");
    assert_eq!(h.wolfram(h.eval("Partition[{1, 2, 3, 4}, 2]")), "{{1, 2}, {3, 4}}");
    assert_eq!(h.wolfram(h.eval("ConstantArray[0, 3]")), "{0, 0, 0}");
}

#[test]
fn union_accumulate_free_q_and_extract() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Union[{1, 2}, {2, 3}]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("Intersection[{1, 2}, {2, 3}]")), "{2}");
    assert_eq!(h.wolfram(h.eval("Accumulate[{1, 2, 3}]")), "{1, 3, 6}");
    assert_eq!(h.wolfram(h.eval("Differences[{1, 4, 9}]")), "{3, 5}");
    assert_eq!(h.wolfram(h.eval("FreeQ[{1, 2}, 3]")), "True");
    assert_eq!(h.wolfram(h.eval("Extract[{1, 2, 3}, 2]")), "2");
}

#[test]
fn accumulate_differences_on_matrix_binding() {
    let h = H::new();
    // Living 16: bound 1×n MatrixRef keeps orientation through Accumulate / Differences.
    assert_eq!(h.wolfram(h.eval("A={1, 2, 3}; Accumulate[A]")), "{1, 3, 6}");
    assert_eq!(h.wolfram(h.eval("B={1, 4, 9}; Differences[B]")), "{3, 5}");
}

#[test]
fn pad_left_riffle_position_and_array() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("PadLeft[{1, 2}, 4]")), "{0, 0, 1, 2}");
    assert_eq!(h.wolfram(h.eval("Riffle[{1, 2}, {a, b}]")), "{1, a, 2, b}");
    assert_eq!(h.wolfram(h.eval("Position[{1, 2, 1}, 1]")), "{{1}, {3}}");
    assert_eq!(h.wolfram(h.eval("Array[f, 3]")), "{f[1], f[2], f[3]}");
}

#[test]
fn total_sums_list_elements() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Total[{1, 2, 3}]")), "6");
}

#[test]
fn total_and_product_on_matrix_binding() {
    let h = H::new();
    // Living 16: Total/Product on bound MatrixRef (column reduce) without nested-list reverse recognition.
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Total[A]")), "{4, 6}");
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {3, 4}}; Product[A]")), "{3, 8}");
}

#[test]
fn matrix_rank_and_tr_on_matrix_binding() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("A={{1, 2}, {2, 4}}; MatrixRank[A]")), "1");
    assert_eq!(h.wolfram(h.eval("B={{1, 2}, {3, 4}}; Tr[B]")), "5");
}

#[test]
fn cases_filters_integer_blank() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Cases[{1, 2, 3}, _Integer]")), "{1, 2, 3}");
    assert_eq!(h.wolfram(h.eval("Cases[{1, a, 2}, _Integer]")), "{1, 2}");
}

#[test]
fn delete_cases_rejects_integer_blank() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("DeleteCases[{1, a, 2}, _Integer]")), "{a}");
    assert_eq!(h.wolfram(h.eval("DeleteCases[{1, 2, 3}, _Integer]")), "{}");
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
fn head_evaluates_args_then_extracts() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("Head[{1, 2}]")), "List");
    assert_eq!(h.wolfram(h.eval("Head[a + b]")), "Plus");
    // Mathematica evaluates args: Head[1+2] → Head[3] → Integer (not HoldFirst → Plus).
    assert_eq!(h.wolfram(h.eval("Head[1 + 2]")), "Integer");
    // Extension head stays Extension identity; dialect render prints the surface name.
    assert_eq!(h.wolfram(h.eval("Head[f[x]]")), "f");
}

#[test]
fn hold_complete_and_unevaluated_preserve_plus() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("HoldComplete[1 + 1]")), "HoldComplete[1 + 1]");
    assert_eq!(h.wolfram(h.eval("Unevaluated[1 + 1]")), "Unevaluated[1 + 1]");
    assert_eq!(h.wolfram(h.eval("Evaluate[HoldComplete[1 + 1]]")), "2");
    assert_eq!(h.wolfram(h.eval("Evaluate[Unevaluated[1 + 1]]")), "2");
    assert_eq!(h.wolfram(h.eval("ReleaseHold[HoldComplete[1 + 1]]")), "2");
}

#[test]
fn unary_minus_binds_looser_than_power() {
    let h = H::new();
    let w = h.parse_w("-x^2");
    assert_eq!(
        w,
        WolframForm::call(
            "Times",
            vec![WolframForm::int(-1), WolframForm::call("Power", vec![WolframForm::symbol("x"), WolframForm::int(2)]),]
        )
    );
    let paren = h.parse_w("(-x)^2");
    assert_eq!(
        paren,
        WolframForm::call(
            "Power",
            vec![WolframForm::call("Times", vec![WolframForm::int(-1), WolframForm::symbol("x")]), WolframForm::int(2),]
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

#[test]
fn indeterminate_forms_fold_to_indeterminate() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("0/0")), "Indeterminate");
    assert_eq!(h.wolfram(h.eval("0^0")), "Indeterminate");
    assert_eq!(h.wolfram(h.eval("Infinity - Infinity")), "Indeterminate");
}

#[test]
fn composition_ops_keep_forms() {
    let h = H::new();
    let right = h.parse_w("f/*g");
    assert_eq!(right.head_name(), Some("RightComposition"));
    assert_eq!(render(&right), "f/*g");
    assert_eq!(h.wolfram(h.eval("f/*g")), "f/*g");

    let left = h.parse_w("g@*f");
    assert_eq!(left.head_name(), Some("Composition"));
    assert_eq!(render(&left), "g@*f");
    assert_eq!(h.wolfram(h.eval("g@*f")), "g@*f");

    // Must not silently collapse to the last symbol.
    assert_ne!(h.wolfram(h.eval("f/*g")), "g");
    assert_ne!(h.wolfram(h.eval("g@*f")), "f");
}

#[test]
fn timing_and_trace_capture_args() {
    let h = H::new();
    // HoldAll-like: must not evaluate the body before wrapping.
    assert_eq!(h.wolfram(h.eval("Timing[1 + 1]")), "Timing[1 + 1]");
    assert_ne!(h.wolfram(h.eval("Timing[1 + 1]")), "Timing[2]");
    assert_eq!(h.wolfram(h.eval("Trace[1 + 1]")), "Trace[1 + 1]");
    assert_ne!(h.wolfram(h.eval("Trace[1 + 1]")), "Trace[2]");
}

#[test]
fn parallel_evaluate_and_input_form_capture_args() {
    let h = H::new();
    assert_eq!(h.wolfram(h.eval("ParallelEvaluate[1 + 1]")), "ParallelEvaluate[1 + 1]");
    assert_ne!(h.wolfram(h.eval("ParallelEvaluate[1 + 1]")), "ParallelEvaluate[2]");
    assert_eq!(h.wolfram(h.eval("InputForm[1 + 1]")), "InputForm[1 + 1]");
    assert_ne!(h.wolfram(h.eval("InputForm[1 + 1]")), "InputForm[2]");
}

#[test]
fn cancel_evaluates_args_then_residuals() {
    let h = H::new();
    // Cancel is not Hold: args evaluate before residual echo (no Cancel kernel yet).
    assert_eq!(h.wolfram(h.eval("Cancel[1 + 1]")), "Cancel[2]");
    let got = h.wolfram(h.eval("Cancel[(x^2 - 1)/(x - 1)]"));
    assert!(got.starts_with("Cancel["), "got {got}");
    // Must not claim algebraic cancelation (`1 + x`) without a Cancel kernel.
    assert_ne!(got, "1 + x");
    assert_ne!(got, "Cancel[1 + x]");
}

#[test]
fn derivative_prime_sugar_forms() {
    let y_prime = parse_mathematica("y'").unwrap();
    assert_eq!(render(&y_prime), "y'");
    match &y_prime {
        WolframForm::Call { head, args } => {
            assert_eq!(args.len(), 1);
            assert!(args[0].is_symbol("y"));
            assert_eq!(head.head_name(), Some("Derivative"));
        }
        other => panic!("expected Derivative[1][y], got {other:?}"),
    }

    let applied = parse_mathematica("y'[x]").unwrap();
    assert_eq!(render(&applied), "y'[x]");
    assert_ne!(render(&applied), "x");

    let h = H::new();
    let dsv = h.wolfram(h.eval("DSolveValue[y'[x] == y[x], y[x], x]"));
    // Must not silently collapse to bare `x` (old SILENT WRONG).
    assert_ne!(dsv, "x");
    assert!(dsv.contains("DSolveValue") || dsv.contains("Derivative") || dsv.contains("y'"), "got {dsv}");
}
