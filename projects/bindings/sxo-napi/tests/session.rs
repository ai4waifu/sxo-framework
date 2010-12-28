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
    assert!(session.structural_eq(session.project_result(e.result_id).unwrap(), seven));
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
    assert!(session.structural_eq(session.project_result(e.result_id).unwrap(), expected));
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
    let d_s = session.render_as_wolfram(session.project_result(d_out.result_id).unwrap());
    assert!(d_s.contains('x'), "got {d_s}");
    // Domain Goal → IR → Result：微积分未准入时为 Candidate，不得静默落成原式成功。
    assert!(matches!(d_out.status.as_str(), "Candidate" | "Exact"), "unexpected status {}", d_out.status);
}

#[test]
fn simplify_wolfram_form_matches_simplify_surface() {
    let session = Session::new();
    let form = session.parse_mathematica("x + 0").unwrap();
    let via_form = session.simplify_wolfram_form(&form).unwrap();
    let via_surface = session.evaluate_mathematica("Simplify[x + 0]").unwrap();
    assert!(session.structural_eq(
        session.project_result(via_form.result_id).unwrap(),
        session.project_result(via_surface.result_id).unwrap()
    ));
    assert_eq!(via_form.status, via_surface.status);
}

#[test]
fn differentiate_wolfram_form_matches_d_surface_lower() {
    let session = Session::new();
    let form = session.parse_mathematica("x^3").unwrap();
    let via_form = session.differentiate_wolfram_form(&form, "x").unwrap();
    let via_surface = session.evaluate_mathematica("D[x^3, x]").unwrap();
    assert!(session.structural_eq(
        session.project_result(via_form.result_id).unwrap(),
        session.project_result(via_surface.result_id).unwrap()
    ));
    assert_eq!(via_form.status, via_surface.status);
}

#[test]
fn differentiate_can_link_derived_from_evaluate_parent() {
    let session = Session::new();
    let evaluated = session.evaluate_mathematica("x^3").unwrap();
    let term = session.project_result(evaluated.result_id).unwrap();
    let d_out = session.differentiate_outcome(term, "x").unwrap();
    assert!(session.link_derived_from(d_out.result_id, evaluated.result_id));
    assert_eq!(session.derived_from(d_out.result_id), Some(evaluated.result_id));
}

#[test]
fn result_transforms_record_derived_from_on_the_same_outcome() {
    let session = Session::new();
    let evaluated = session.evaluate_mathematica("x^3").unwrap();
    let derived = session.differentiate_result(evaluated.result_id, "x").unwrap();
    assert_eq!(session.derived_from(derived.result_id), Some(evaluated.result_id));
    assert!(!derived.status.is_empty());
    assert!(!derived.coverage.is_empty());
    let via_surface = session.evaluate_mathematica("D[x^3, x]").unwrap();
    assert!(session.structural_eq(session.project_result(derived.result_id).unwrap(), session.project_result(via_surface.result_id).unwrap()));
    assert_ne!(session.render_as_wolfram(session.project_result(derived.result_id).unwrap()), "Null");

    let sum = session.evaluate_matlab("sin(x)^2 + cos(x)^2").unwrap();
    let simplified = session.simplify_result(sum.result_id).unwrap();
    assert_eq!(session.derived_from(simplified.result_id), Some(sum.result_id));
    assert_ne!(simplified.result_id, sum.result_id, "simplify must publish a new ResultId");
    assert_eq!(session.render_as_matlab(session.project_result(simplified.result_id).unwrap()), "1");
    assert!(!simplified.status.is_empty());
    assert!(!simplified.coverage.is_empty());
}

#[test]
fn project_conditions_exposes_result_predicates() {
    use athena::{
        runtime::results::{ComputationResult, CoverageStatus},
        types::{ComputationStatus, Condition, Predicate},
    };

    let session = Session::new();
    let result_id = session.with_math_mut(|s| {
        let term = push_int(s, 1);
        let symbol = s.arena.symbols_mut().intern("x");
        let result = ComputationResult::with_status(ComputationStatus::Conditional, CoverageStatus::Partial)
            .with_symbolic_term(term)
            .with_condition(Condition { predicate: Predicate::SymbolReal(symbol), resolved: false });
        s.insert_result(result)
    });
    let conditions = session.project_conditions(result_id);
    assert_eq!(conditions, vec!["SymbolReal resolved=false".to_string()]);
    assert!(session.project_diagnostics(result_id).is_empty());
    assert!(session.project_provider(result_id).is_none());
}

#[test]
fn project_provider_exposes_result_stamp() {
    use athena::{
        runtime::results::{ComputationResult, CoverageStatus, ResultProviderId},
        types::ComputationStatus,
    };

    let session = Session::new();
    let result_id = session.with_math_mut(|s| {
        let term = push_int(s, 1);
        let result = ComputationResult::with_status(ComputationStatus::Exact, CoverageStatus::Full)
            .with_symbolic_term(term)
            .with_provider(ResultProviderId::LINEAR_ALGEBRA);
        s.insert_result(result)
    });
    assert_eq!(session.project_provider(result_id), Some(format!("LinearAlgebra@v{}", ResultProviderId::CONTRACT_VERSION)));
}

#[test]
fn project_evidence_exposes_trusted_kernel_summary() {
    use athena::{
        runtime::results::{ComputationResult, CoverageStatus, ResultEvidence, ResultProviderId},
        types::ComputationStatus,
    };

    let session = Session::new();
    let result_id = session.with_math_mut(|s| {
        let term = push_int(s, 1);
        let result = ComputationResult::with_status(ComputationStatus::Candidate, CoverageStatus::Full)
            .with_symbolic_term(term)
            .with_provider(ResultProviderId::SOLVE)
            .with_evidence(ResultEvidence::TrustedKernelSummary {
                provider: ResultProviderId::SOLVE,
                summary: "solution_rules coverage=Complete".into(),
            });
        s.insert_result(result)
    });
    assert_eq!(
        session.project_evidence(result_id),
        vec!["TrustedKernelSummary provider=Solve solution_rules coverage=Complete".to_string()]
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
fn napi_path_hold_complete_flatten_and_indeterminate_with_simplify() {
    // Mirrors TS feature-matrix: evaluate + strategy=simplify (autoSimplify).
    let session = Session::new();
    let hold = session.evaluate_mathematica("HoldComplete[1 + 1]").unwrap();
    let hold_term = session.project_result(hold.result_id).unwrap();
    let hold_simplified = session.simplify_outcome(hold_term).unwrap();
    assert_eq!(
        session.render_as_wolfram(session.project_result(hold_simplified.result_id).unwrap()),
        "HoldComplete[1 + 1]",
        "autoSimplify must not evaluate inside HoldComplete"
    );

    let uneval = session.evaluate_mathematica("Unevaluated[1 + 1]").unwrap();
    assert_eq!(session.render_as_wolfram(session.project_result(uneval.result_id).unwrap()), "Unevaluated[1 + 1]");

    let flat = session.evaluate_mathematica("Flatten[{{1, 2}, {3, 4}}]").unwrap();
    assert_eq!(
        session.render_as_wolfram(session.project_result(flat.result_id).unwrap()),
        "{1, 2, 3, 4}",
        "Flatten must execute, not residual Flatten[…]"
    );

    for (input, expect) in [("0/0", "Indeterminate"), ("Infinity - Infinity", "Indeterminate"), ("0^0", "Indeterminate")] {
        let out = session.evaluate_mathematica(input).unwrap();
        let text = session.render_as_wolfram(session.project_result(out.result_id).unwrap());
        assert_eq!(text, expect, "input={input}");
    }
}

#[test]
fn session_set_persists_across_mathematica_evaluates() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let six = session.with_math_mut(|s| push_int(s, 6));
    assert!(
        session.structural_eq(session.project_result(session.evaluate_mathematica("x = 5").unwrap().result_id).unwrap(), five)
    );
    assert!(
        session.structural_eq(session.project_result(session.evaluate_mathematica("x + 1").unwrap().result_id).unwrap(), six)
    );
    session.clear_definitions();
    let cleared = session.evaluate_mathematica("x + 1").unwrap();
    let cleared_term = session.project_result(cleared.result_id).unwrap();
    let text = session.with_math(|s| term_debug(s, cleared_term));
    assert!(text.contains("Plus") || text.contains("Add") || text.contains('+'), "expected free Plus after clear, got {text}");
}

#[test]
fn session_set_persists_across_matlab_evaluates() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let six = session.with_math_mut(|s| push_int(s, 6));
    assert!(session.structural_eq(session.project_result(session.evaluate_matlab("x = 5").unwrap().result_id).unwrap(), five));
    assert!(session.structural_eq(session.project_result(session.evaluate_matlab("x + 1").unwrap().result_id).unwrap(), six));
}

#[test]
fn session_setdelayed_evaluates_on_use() {
    let session = Session::new();
    let null = session.evaluate_mathematica("a := 1 + 1").unwrap();
    let null_term = session.project_result(null.result_id).unwrap();
    session.with_math(|s| {
        assert!(matches!(s.arena.get(null_term), Some(TermNode::Atom(Atom::Null))));
    });
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("a").unwrap().result_id).unwrap(), two));
}

#[test]
fn module_does_not_clobber_session_binding() {
    let session = Session::new();
    let five = session.with_math_mut(|s| push_int(s, 5));
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(
        session.structural_eq(session.project_result(session.evaluate_mathematica("x = 5").unwrap().result_id).unwrap(), five)
    );
    assert!(session.structural_eq(
        session.project_result(session.evaluate_mathematica("Module[{x = 1}, x + 1]").unwrap().result_id).unwrap(),
        two
    ));
    assert!(session.structural_eq(session.project_result(session.evaluate_mathematica("x").unwrap().result_id).unwrap(), five));
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
            Dialect::Matlab => session.render_as_matlab(session.project_result(out.result_id).unwrap()),
            _ => session.render_as_wolfram(session.project_result(out.result_id).unwrap()),
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
        ("[1, 2; 3, 4].'", "[1, 3; 2, 4]"),
        ("[1, 2; 3, 4]'", "[1, 3; 2, 4]"),
        ("[1+2i, 3; 4, 5]'", "[1 - 2*i, 4; 3, 5]"),
        ("[1+i, 0; 0, 1-i]*[1, i; -i, 1]", "[1 + i, -1 + i; -1 - i, 1 - i]"),
        ("[1+i, 2; 3, 4].'", "[1 + i, 3; 2, 4]"),
        ("tril([1+i, 2; 3, 4-i])", "[1 + i, 0; 3, 4 - i]"),
        ("[1+i, 2; 3, 4](1,:)", "[1 + i, 2]"),
        ("kron([1+i], [1, i])", "[1 + i, -1 + i]"),
        ("M=[1, 2; 3, 4]; M(:, 2)=[9; 8]; M", "[1, 9; 3, 8]"),
        ("M=[1, 2; 3, 4]; M(1, :)=[9, 8]; M", "[9, 8; 3, 4]"),
        ("A=zeros(2); A(3, 3)=1; A", "[0, 0, 0; 0, 0, 0; 0, 0, 1]"),
        ("B=1:4; B(end+1)=5; B", "[1, 2, 3, 4, 5]"),
        ("A=[1, 2; 3, 4]; A(2)", "3"),
        ("A=[1, 2; 3, 4]; A(1, 2)", "2"),
        ("[1, 2; 3, 4](:)", "[1; 3; 2; 4]"),
        ("[1, 2; 3, 4].*[5, 6; 7, 8]", "[5, 12; 21, 32]"),
        ("[1, 2; 3, 4]*[5, 6; 7, 8]", "[19, 22; 43, 50]"),
        ("[6, 8; 10, 12]./[2, 4; 5, 6]", "[3, 2; 2, 2]"),
        ("[1, 2; 3, 4]/[1, 2; 3, 4]", "[1, 0; 0, 1]"),
        ("inv([1, 2; 2, 4])", "inv(Singular)"),
        ("cond([1, 2; 2, 4])", "inf"),
        ("[1, 2; 3, 4] \\ [5; 6]", "[-4; 9/2]"),
        ("[1, 2; 2, 4] \\ [1; 0]", "linsolve(Inconsistent)"),
        ("[1, 2; 2, 4] \\ [2; 4]", "linsolve(Infinite, 1)"),
        ("[1, 0] / [1, 2; 2, 4]", "linsolve(Inconsistent)"),
        ("[1, 2; 2, 4] / [1, 2; 2, 4]", "linsolve(Infinite, 1)"),
        ("0/0", "NaN"),
        ("Inf - Inf", "NaN"),
        ("0.0/0.0", "NaN"),
        ("sum([1, 2, 3])", "6"),
        ("sum([1, 2; 3, 4])", "[4, 6]"),
        ("sum([1, 2; 3, 4], 2)", "[3; 7]"),
        ("prod([2, 3, 4])", "24"),
        ("prod([1, 2; 3, 4])", "[3, 8]"),
        ("[1, 2, 3](end)", "3"),
    ];
    for (input, expected) in cases {
        let direct_session = Session::new();
        let direct = direct_session.evaluate_matlab(input).unwrap();
        let direct_text = direct_session.render_as_matlab(direct_session.project_result(direct.result_id).unwrap());

        let handle_session = Session::new();
        let form = handle_session.parse_matlab_form(input).unwrap();
        let via_handle = handle_session.evaluate_matlab_form(&form).unwrap();
        let handle_text = handle_session.render_as_matlab(handle_session.project_result(via_handle.result_id).unwrap());

        assert_eq!(direct_text, handle_text, "parity failed for {input}: direct={direct_text} handle={handle_text}");
        assert_eq!(direct_text, expected, "expected value for {input}");
        assert_eq!(direct.status, via_handle.status, "status parity for {input}");
        assert_eq!(direct.coverage, via_handle.coverage, "coverage parity for {input}");
        assert_ne!(direct_text, "Null", "projection must not invent Null for {input}");
    }
}

/// Direct string evaluate and parse→Form→evaluate must agree on MMA matrix surfaces.
#[test]
fn mathematica_direct_and_handle_matrix_parity() {
    let cases = [
        ("Transpose[{{1, 2}, {3, 4}}]", "{{1, 3}, {2, 4}}"),
        ("ConjugateTranspose[{{1, 2}, {3, 4}}]", "{{1, 3}, {2, 4}}"),
        ("ConjugateTranspose[{{1 + 2 I, 3}, {4, 5}}]", "{{1 - 2*I, 4}, {3, 5}}"),
        ("Dot[{{1 + I, 0}, {0, 1 - I}}, {{1, I}, {-I, 1}}]", "{{1 + I, -1 + I}, {-1 - I, 1 - I}}"),
        ("Transpose[{{1 + I, 2}, {3, 4}}]", "{{1 + I, 3}, {2, 4}}"),
        ("LowerTriangularize[{{1 + I, 2}, {3, 4 - I}}]", "{{1 + I, 0}, {3, 4 - I}}"),
        ("Reverse[{{1 + I, 2}, {3, 4}}]", "{{3, 4}, {1 + I, 2}}"),
        ("KroneckerProduct[{{1 + I}}, {{1, I}}]", "{1 + I, -1 + I}"),
        ("A={{1, 2}, {3, 4}}; ReplacePart[A, {1, 2} -> 9]; A", "{{1, 9}, {3, 4}}"),
        ("SymmetricMatrixQ[{{1, 2}, {2, 1}}]", "1"),
        ("SymmetricMatrixQ[{{1, 2}, {3, 4}}]", "0"),
        ("Inverse[{{1, 2}, {3, 4}}]", "{{-2, 1}, {3/2, -1/2}}"),
        ("Part[{{1, 2}, {3, 4}}, 1, 2]", "2"),
        ("0/0", "Indeterminate"),
        ("(1/0)-(1/0)", "Indeterminate"),
        ("ReplacePart[{1, 2, 3}, 2 -> 9]", "{1, 9, 3}"),
        ("Infinity - Infinity", "Indeterminate"),
        ("0.0/0.0", "Indeterminate"),
        ("Dot[{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}]", "{{19, 22}, {43, 50}}"),
        ("{{1, 2}, {3, 4}}*{{5, 6}, {7, 8}}", "{{5, 12}, {21, 32}}"),
        ("LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]", "{{-4}, {9/2}}"),
        ("LinearSolve[{{1, 2}, {2, 4}}, {{1}, {0}}]", "LinearSolve[Inconsistent]"),
        ("LinearSolve[{{1, 2}, {2, 4}}, {{2}, {4}}]", "LinearSolve[Infinite, 1]"),
        ("Total[{1, 2, 3}]", "6"),
        ("Total[{{1, 2}, {3, 4}}]", "{4, 6}"),
        ("Total[{{1, 2}, {3, 4}}, {2}]", "{{3}, {7}}"),
        ("Transpose[{{1, 2}, {3}}]", "Transpose[{{1, 2}, {3}}]"),
        ("Inverse[{{1, 2}, {2, 4}}]", "Inverse[Singular]"),
        ("MapAt[f, {1, 2, 3}, 2]", "MapAt[f, {1, 2, 3}, 2]"),
    ];
    for (input, expected) in cases {
        let direct_session = Session::new();
        let direct = direct_session.evaluate_mathematica(input).unwrap();
        let direct_text = direct_session.render_as_wolfram(direct_session.project_result(direct.result_id).unwrap());

        let handle_session = Session::new();
        let form = handle_session.parse_mathematica(input).unwrap();
        let via_handle = handle_session.evaluate_wolfram_form(&form).unwrap();
        let handle_text = handle_session.render_as_wolfram(handle_session.project_result(via_handle.result_id).unwrap());

        assert_eq!(direct_text, handle_text, "parity failed for {input}: direct={direct_text} handle={handle_text}");
        assert_eq!(direct_text, expected, "expected value for {input}");
        assert_eq!(direct.status, via_handle.status, "status parity for {input}");
        assert_eq!(direct.coverage, via_handle.coverage, "coverage parity for {input}");
        assert_ne!(direct_text, "Null", "projection must not invent Null for {input}");
    }
}

#[test]
fn evaluate_keeps_result_id_and_projects_on_demand() {
    let session = Session::new();
    let out = session.evaluate_matlab("1+1").unwrap();
    assert_eq!(out.status, "Exact");
    let two = session.with_math_mut(|s| push_int(s, 2));
    assert!(session.structural_eq(session.project_result(out.result_id).unwrap(), two));
}

#[test]
fn evaluate_then_simplify_trig_identity_in_one_session() {
    let session = Session::new();
    let out = session.evaluate_matlab("sin(x)^2 + cos(x)^2").unwrap();
    let term = session.project_result(out.result_id).unwrap();
    let simplified = session.simplify_outcome(term).unwrap();
    assert!(session.link_derived_from(simplified.result_id, out.result_id));
    let simplified_term = session.project_result(simplified.result_id).unwrap();
    assert_eq!(session.render_as_matlab(simplified_term), "1");
    assert_ne!(simplified.result_id, out.result_id, "simplify must publish a new ResultId");
    assert_eq!(session.derived_from(simplified.result_id), Some(out.result_id));
    assert!(!simplified.status.is_empty());
    assert!(!simplified.coverage.is_empty());
}

#[test]
fn simplify_strategy_does_not_rebind_block_free_symbol() {
    let session = Session::new();
    session.evaluate_mathematica("x = 5").unwrap();
    let blocked = session.evaluate_mathematica("Block[{x}, x]").unwrap();
    let term = session.project_result(blocked.result_id).unwrap();
    let simplified = session.simplify_outcome(term).unwrap();
    let text = session.render_as_wolfram(session.project_result(simplified.result_id).unwrap());
    assert_eq!(text, "x", "autoSimplify must not turn Block free x into ambient Own 5, got {text}");
}

#[test]
fn compound_block_clear_with_simplify_strategy_stays_free() {
    // Mirrors Mathematica.create({ autoSimplify: true }).evaluate("x = 5; Block[{x}, x]").
    let session = Session::new();
    let blocked = session.evaluate_mathematica("x = 5; Block[{x}, x]").unwrap();
    let term = session.project_result(blocked.result_id).unwrap();
    assert_eq!(
        session.render_as_wolfram(term),
        "x",
        "compound Block clear must yield free x before simplify, got {}",
        session.render_as_wolfram(term)
    );
    let simplified = session.simplify_outcome(term).unwrap();
    let text = session.render_as_wolfram(session.project_result(simplified.result_id).unwrap());
    assert_eq!(text, "x", "simplify strategy must keep free x, got {text}");
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
