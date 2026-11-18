//! Result display never substitutes source text for an Athena projection.

use science::{
    CasEngine, Session,
    api::CasRequest,
    ir::{ApplicationHead, Atom, SemanticOperator, TermNode},
    numeric::number_from_wire,
    runtime::results::{ComputationResult, CoverageStatus},
    types::{CollectionKind, ComputationStatus, ResultId, SourceSpan, TermId, WireNumber},
};
use sxo_dialect_maple::{MapleForm, MapleOperator, MapleRenderError, lower_request, render_maple_result_projection};

fn project(session: &mut Session, term: TermId) -> ResultId {
    let mut result = ComputationResult::with_status(ComputationStatus::Unknown, CoverageStatus::Unknown);
    result.symbolic_term = Some(term);
    session.results.insert(result)
}

#[test]
fn exact_result_display_reads_computed_value_without_mutation() {
    for (numerator, denominator, expected) in
        [("1", "3", "1/3"), ("123456789012345678901234567890", "1", "123456789012345678901234567890")]
    {
        let mut session = Session::new();
        let form = MapleForm::Binary {
            operator: MapleOperator::Divide,
            left: Box::new(MapleForm::Integer(numerator.to_owned())),
            right: Box::new(MapleForm::Integer(denominator.to_owned())),
        };
        let request = lower_request(&mut session, &form).unwrap();
        let result_id = CasEngine::new().execute_request(&mut session, request).unwrap();
        let before_terms = session.arena.len();
        let before_result = format!("{:?}", session.results.get(result_id).unwrap());
        assert_eq!(render_maple_result_projection(&session, result_id).unwrap(), expected);
        assert_eq!(session.arena.len(), before_terms);
        assert_eq!(format!("{:?}", session.results.get(result_id).unwrap()), before_result);
        let result = session.results.get(result_id).unwrap();
        assert_eq!(result.status, ComputationStatus::Exact);
        assert_eq!(result.coverage, CoverageStatus::Full);
        assert!(result.conditions.is_empty());
        assert!(result.diagnostics.is_empty());
    }
}

#[test]
fn missing_results_and_projections_never_render_null() {
    let mut session = Session::new();
    let missing = ResultId(u32::MAX);
    assert_eq!(render_maple_result_projection(&session, missing), Err(MapleRenderError::UnknownResult(missing)));
    let id = session.results.insert(ComputationResult::with_status(ComputationStatus::Unknown, CoverageStatus::Unsupported));
    assert_eq!(render_maple_result_projection(&session, id), Err(MapleRenderError::MissingSymbolicProjection(id)));
    let dangling = TermId(u32::MAX);
    let id = project(&mut session, dangling);
    assert_eq!(render_maple_result_projection(&session, id), Err(MapleRenderError::UnknownTerm(dangling)));
}

#[test]
fn nested_lists_display_without_matrix_inference_or_status_promotion() {
    let mut session = Session::new();
    let form = MapleForm::List(vec![MapleForm::List(vec![MapleForm::Integer("1".to_owned())]), MapleForm::List(vec![])]);
    let CasRequest::Term(term) = lower_request(&mut session, &form).unwrap()
    else {
        panic!("list term")
    };
    let id = project(&mut session, term);
    assert_eq!(render_maple_result_projection(&session, id).unwrap(), "[[1], []]");
    let result = session.results.get(id).unwrap();
    assert_eq!(result.status, ComputationStatus::Unknown);
    assert_eq!(result.coverage, CoverageStatus::Unknown);
}

#[test]
fn projected_typed_arithmetic_is_display_only() {
    let mut session = Session::new();
    for (operator, expected) in [
        (MapleOperator::Add, "((x) + (2))"),
        (MapleOperator::Subtract, "((x) - (2))"),
        (MapleOperator::Multiply, "((x) * (2))"),
        (MapleOperator::Divide, "((x) / (2))"),
        (MapleOperator::Power, "((x) ^ (2))"),
    ] {
        let form = MapleForm::Binary {
            operator,
            left: Box::new(MapleForm::Symbol("x".to_owned())),
            right: Box::new(MapleForm::Integer("2".to_owned())),
        };
        let CasRequest::Term(term) = lower_request(&mut session, &form).unwrap()
        else {
            panic!("arithmetic term")
        };
        let id = project(&mut session, term);
        assert_eq!(render_maple_result_projection(&session, id).unwrap(), expected);
    }
}

#[test]
fn unsupported_atoms_collections_and_operators_are_explicit_errors() {
    let mut session = Session::new();
    for node in [
        TermNode::Atom(Atom::Null),
        TermNode::Atom(Atom::Boolean(true)),
        TermNode::Collection { kind: CollectionKind::Matrix, elements: vec![] },
        TermNode::Collection { kind: CollectionKind::StructuralSequence, elements: vec![] },
        TermNode::Collection { kind: CollectionKind::OrderedCollection, elements: vec![TermId(u32::MAX)] },
        TermNode::Application { head: ApplicationHead::Semantic(SemanticOperator::Add), arguments: vec![] },
        TermNode::Application { head: ApplicationHead::Semantic(SemanticOperator::Equal), arguments: vec![] },
    ] {
        let term = session.arena.push(node, SourceSpan::default());
        let id = project(&mut session, term);
        assert!(render_maple_result_projection(&session, id).is_err());
    }
}

#[test]
fn rational_and_negative_power_bases_are_explicitly_grouped() {
    let mut session = Session::new();
    for (wire, expected) in
        [(WireNumber::rational_i64(1, 3).unwrap(), "((1/3) ^ (2))"), (WireNumber::small_int(-2), "((-2) ^ (2))")]
    {
        let base = number_from_wire(&wire).unwrap();
        let base = session.arena.push(TermNode::Atom(Atom::Number(base)), SourceSpan::default());
        let exponent = number_from_wire(&WireNumber::small_int(2)).unwrap();
        let exponent = session.arena.push(TermNode::Atom(Atom::Number(exponent)), SourceSpan::default());
        let power = session.arena.push(
            TermNode::Application { head: ApplicationHead::Semantic(SemanticOperator::Power), arguments: vec![base, exponent] },
            SourceSpan::default(),
        );
        let id = project(&mut session, power);
        assert_eq!(render_maple_result_projection(&session, id).unwrap(), expected);
    }
}

#[test]
fn approximate_numbers_and_extension_heads_are_not_silently_admitted() {
    let mut session = Session::new();
    let number = number_from_wire(&WireNumber::machine(0.125)).unwrap();
    let term = session.arena.push(TermNode::Atom(Atom::Number(number)), SourceSpan::default());
    let id = project(&mut session, term);
    assert_eq!(render_maple_result_projection(&session, id), Err(MapleRenderError::Unsupported("non-exact number")));
    let head = session.extensions.intern("solve");
    let term = session
        .arena
        .push(TermNode::Application { head: ApplicationHead::Extension(head), arguments: vec![] }, SourceSpan::default());
    let id = project(&mut session, term);
    assert_eq!(render_maple_result_projection(&session, id), Err(MapleRenderError::Unsupported("extension head")));
}
