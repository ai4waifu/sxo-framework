//! Host integration tests across dialect crates and Athena.

use athena::{
    api::{AthenaRequest, DomainGoal},
    diagnostics::term_summary::term_debug,
    domains::DomainRequest,
    domains::calculus::CalculusRequest,
    ir::Atom,
    ir::TermNode,
    runtime::values::arena::push_int,
};
use sxo_dialect_mathematica::{WExpr, parse_number_literal};
use sxo_napi::session::Session;
use sxo_types::Dialect;

#[test]
fn math_evaluate_arith() {
    let session = Session::new();
    let e = session.evaluate_mathematica("1 + 2 * 3").unwrap();
    let seven = session.with_math_mut(|s| push_int(s, 7));
    assert!(session.structural_eq(e, seven));
}

#[test]
fn wexpr_roundtrip_via_session() {
    let session = Session::new();
    let w = WExpr::call("Sin", vec![WExpr::symbol("x")]);
    let t = session.lower_mathematica(&w);
    assert_eq!(session.to_mathematica(t), w);
}

#[test]
fn big_integer_arithmetic() {
    let session = Session::new();
    let e = session.evaluate_mathematica("99999999999999999999 + 1").unwrap();
    let expected_n = parse_number_literal("100000000000000000000").unwrap();
    let expected = session.with_math_mut(|s| {
        s.arena
            .push(TermNode::Atom(Atom::Number(athena::runtime::values::numeric_clone::clone_number(&expected_n))), athena::types::SourceSpan::default())
    });
    assert!(session.structural_eq(e, expected));
}

#[test]
fn bridge_lowers_to_kernel_app() {
    let session = Session::new();
    let w = session.parse_mathematica("1 + 2").unwrap();
    let t = session.lower_mathematica(&w);
    session.with_math(|s| {
        assert!(matches!(s.arena.get(t), Some(TermNode::Application { .. })));
    });
}

#[test]
fn dialect_d_limit_series_lower_to_domain() {
    let session = Session::new();
    let w = session.parse_mathematica("D[x^3, x]").unwrap();
    let request = session.with_math_mut(|s| sxo_dialect_mathematica::lower_request(s, &w));
    assert!(matches!(
        request,
        AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::Calculus(CalculusRequest::Derivative { .. })))
    ));
    let d_out = session.evaluate_mathematica("D[x^3, x]").unwrap();
    let d_s = session.render_as_wolfram(d_out);
    assert!(d_s.contains('x'), "got {d_s}");
}

#[test]
fn session_set_persists_across_mathematica_evaluates() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let six = session.with_math_mut(|s| push_int(s, 6));
    assert!(session.structural_eq(session.evaluate_mathematica("x = 5").unwrap(), five));
    assert!(session.structural_eq(session.evaluate_mathematica("x + 1").unwrap(), six));
    session.clear_definitions();
    let cleared = session.evaluate_mathematica("x + 1").unwrap();
    let text = session.with_math(|s| term_debug(s, cleared));
    assert!(text.contains("Plus") || text.contains("Add") || text.contains('+'), "expected free Plus after clear, got {text}");
}

#[test]
fn session_set_persists_across_matlab_evaluates() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let six = session.with_math_mut(|s| push_int(s, 6));
    assert!(session.structural_eq(session.evaluate_matlab("x = 5").unwrap(), five));
    assert!(session.structural_eq(session.evaluate_matlab("x + 1").unwrap(), six));
}

#[test]
fn session_setdelayed_evaluates_on_use() {
    let session = Session::new();
    let null = session.evaluate_mathematica("a := 1 + 1").unwrap();
    session.with_math(|s| {
        assert!(matches!(s.arena.get(null), Some(TermNode::Atom(Atom::Null))));
    });
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.evaluate_mathematica("a").unwrap(), two));
}

#[test]
fn module_does_not_clobber_session_binding() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.evaluate_mathematica("x = 5").unwrap(), five));
    assert!(session.structural_eq(session.evaluate_mathematica("Module[{x = 1}, x + 1]").unwrap(), two));
    assert!(session.structural_eq(session.evaluate_mathematica("x").unwrap(), five));
}

#[test]
fn try_plot_svg_mathematica() {
    let session = Session::new();
    let w = session.parse_mathematica("Plot[x^2, {x, -1, 1}]").unwrap();
    let id = session.lower_mathematica(&w);
    let svg = session.try_plot_svg(id, Dialect::Mathematica).expect("extract").expect("render");
    assert!(svg.contains("<svg"), "{svg}");
}
