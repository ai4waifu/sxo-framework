//! Host integration tests across dialect crates and Athena.

use athena::{
    api::{AthenaRequest, DomainGoal},
    diagnostics::term_summary::term_debug,
    domains::{DomainRequest, calculus::CalculusRequest},
    ir::{Atom, TermNode},
    runtime::values::arena::push_int,
};
use sxo_dialect_mathematica::{WolframForm, parse_number_literal};
use sxo_napi::session::Session;
use sxo_types::Dialect;

#[test]
fn math_evaluate_arith() {
    let session = Session::new();
    let e = session.evaluate_mathematica("1 + 2 * 3").unwrap();
    assert_eq!(e.status, "Exact");
    assert_eq!(e.coverage, "Full");
    let seven = session.with_math_mut(|s| push_int(s, 7));
    assert!(session.structural_eq(session.project_result(e.result_id), seven));
}

#[test]
fn wexpr_roundtrip_via_session() {
    let session = Session::new();
    let w = WolframForm::call("Sin", vec![WolframForm::symbol("x")]);
    let t = session.lower_mathematica(&w);
    assert_eq!(session.to_mathematica(t), w);
}

#[test]
fn big_integer_arithmetic() {
    let session = Session::new();
    let e = session.evaluate_mathematica("99999999999999999999 + 1").unwrap();
    assert_eq!(e.status, "Exact");
    assert_eq!(e.coverage, "Full");
    let expected_n = parse_number_literal("100000000000000000000").unwrap();
    let expected = session.with_math_mut(|s| {
        s.arena.push(
            TermNode::Atom(Atom::Number(athena::runtime::values::numeric_clone::clone_number(&expected_n))),
            athena::types::SourceSpan::default(),
        )
    });
    assert!(session.structural_eq(session.project_result(e.result_id), expected));
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
    let d_s = session.render_as_wolfram(session.project_result(d_out.result_id));
    assert!(d_s.contains('x'), "got {d_s}");
    // Domain Goal → IR → Result：微积分未准入时为 Candidate，不得静默落成原式成功。
    assert!(
        matches!(d_out.status.as_str(), "Candidate" | "Exact"),
        "unexpected status {}",
        d_out.status
    );
}

#[test]
fn residual_unevaluated_is_not_exact_full() {
    let session = Session::new();
    let out = session.evaluate_mathematica("Cos[x]").unwrap();
    assert_ne!(out.status, "Exact", "unevaluated Cos[x] must not claim Exact");
    assert_ne!(out.coverage, "Full", "unevaluated Cos[x] must not claim Full coverage");
}

#[test]
fn session_set_persists_across_mathematica_evaluates() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let six = session.with_math_mut(|s| push_int(s, 6));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("x = 5").unwrap().result_id), five));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("x + 1").unwrap().result_id), six));
    session.clear_definitions();
    let cleared = session.evaluate_mathematica("x + 1").unwrap();
    let cleared_term = session.project_result(cleared.result_id);
    let text = session.with_math(|s| term_debug(s, cleared_term));
    assert!(text.contains("Plus") || text.contains("Add") || text.contains('+'), "expected free Plus after clear, got {text}");
}

#[test]
fn session_set_persists_across_matlab_evaluates() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let six = session.with_math_mut(|s| push_int(s, 6));
    assert!(session.structural_eq(session.project_result(session.evaluate_matlab("x = 5").unwrap().result_id), five));
    assert!(session.structural_eq(session.project_result(session.evaluate_matlab("x + 1").unwrap().result_id), six));
}

#[test]
fn session_setdelayed_evaluates_on_use() {
    let session = Session::new();
    let null = session.evaluate_mathematica("a := 1 + 1").unwrap();
    let null_term = session.project_result(null.result_id);
    session.with_math(|s| {
        assert!(matches!(s.arena.get(null_term), Some(TermNode::Atom(Atom::Null))));
    });
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("a").unwrap().result_id), two));
}

#[test]
fn module_does_not_clobber_session_binding() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("x = 5").unwrap().result_id), five));
    assert!(session.structural_eq(
        session.project_result(session.evaluate_mathematica("Module[{x = 1}, x + 1]").unwrap().result_id),
        two
    ));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("x").unwrap().result_id), five));
}

#[test]
fn probe_eval_forms() {
    let session = Session::new();
    for (input, dialect) in [
        ("diff(x^3, x)", Dialect::Matlab),
        ("int(x^2, x)", Dialect::Matlab),
        ("Hold[1+1]", Dialect::Mathematica),
        ("Solve[x^2 == 1, x]", Dialect::Mathematica),
        ("Integrate[x^2, x]", Dialect::Mathematica),
        ("0 < 1 < 2", Dialect::Mathematica),
        ("1 < 3 > 2", Dialect::Mathematica),
        ("x^2 == 1", Dialect::Mathematica),
        ("Equal[Power[x, 2], 1]", Dialect::Mathematica),
    ] {
        let out = session.evaluate_input(input, dialect).unwrap();
        let kind = match dialect {
            Dialect::Matlab => {
                let form = session.parse_matlab_form(input).unwrap();
                session.with_math_mut(|s| sxo_dialect_matlab::lower_request(s, &form).kind_name().to_string())
            }
            _ => {
                let w = session.parse_mathematica(input).unwrap();
                session.with_math_mut(|s| sxo_dialect_mathematica::lower_request(s, &w).kind_name().to_string())
            }
        };
        let rendered = match dialect {
            Dialect::Matlab => session.render_as_matlab(session.project_result(out.result_id)),
            _ => session.render_as_wolfram(session.project_result(out.result_id)),
        };
        eprintln!("IN={input} kind={kind} status={} coverage={} out={rendered}", out.status, out.coverage);
    }
}

#[test]
fn try_plot_svg_mathematica() {
    let session = Session::new();
    let w = session.parse_mathematica("Plot[x^2, {x, -1, 1}]").unwrap();
    let id = session.lower_mathematica(&w);
    let svg = session.try_plot_svg(id, Dialect::Mathematica).expect("extract").expect("render");
    assert!(svg.contains("<svg"), "{svg}");
}

/// Direct string evaluate and parse→Form→evaluate must agree (R-2.11 / Living 17).
///
/// These cases previously diverged when MATLAB handles re-parsed display text.
#[test]
fn matlab_direct_and_handle_evaluate_parity() {
    let cases = [
        ("(1+2)*3", "9"),
        ("1/(2+3)", "1/5"),
        ("1-(2-3)", "2"),
        ("[1,2].*(3+4)", "[7, 14]"),
    ];
    for (input, expected) in cases {
        let direct_session = Session::new();
        let direct = direct_session.evaluate_matlab(input).unwrap();
        let direct_text = direct_session.render_as_matlab(direct_session.project_result(direct.result_id));

        let handle_session = Session::new();
        let form = handle_session.parse_matlab_form(input).unwrap();
        let via_handle = handle_session.evaluate_matlab_form(&form).unwrap();
        let handle_text = handle_session.render_as_matlab(handle_session.project_result(via_handle.result_id));

        assert_eq!(
            direct_text, handle_text,
            "parity failed for {input}: direct={direct_text} handle={handle_text}"
        );
        assert_eq!(direct_text, expected, "expected value for {input}");
        assert_eq!(direct.status, via_handle.status, "status parity for {input}");
        assert_eq!(direct.coverage, via_handle.coverage, "coverage parity for {input}");
    }
}

#[test]
fn evaluate_keeps_result_id_and_projects_on_demand() {
    let session = Session::new();
    let out = session.evaluate_matlab("1+1").unwrap();
    assert_eq!(out.status, "Exact");
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.project_result(out.result_id), two));
}

#[test]
fn matlab_form_display_without_arena_materialize() {
    use sxo_dialect_matlab::render_matlab_form;

    let form = sxo_dialect_matlab::parse_matlab_form("1/(2+3)").unwrap();
    let text = render_matlab_form(&form);
    assert!(text.contains('2') && text.contains('3'), "got {text}");
    // Display path must not require Session arena writes.
    let session = Session::new();
    let before = session.with_math(|ms| ms.arena.len());
    let _ = render_matlab_form(&form);
    let after = session.with_math(|ms| ms.arena.len());
    assert_eq!(before, after, "Form renderer must not grow the arena");
}
