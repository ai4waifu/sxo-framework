use super::Session;
use sxo_dialect_maple::{MapleForm, MapleOperator, MapleProgram, MapleSpan};

fn fraction() -> MapleForm {
    MapleForm::Binary {
        operator: MapleOperator::Divide,
        left: Box::new(MapleForm::Integer("1".to_owned())),
        right: Box::new(MapleForm::Integer("3".to_owned())),
    }
}

#[test]
fn maple_host_direct_and_materialized_result_parity() {
    let session = Session::new();
    let direct = session.evaluate_maple_form(&fraction()).unwrap();
    let term = session.lower_maple_form(&fraction()).unwrap();
    let materialized = session.evaluate_term_outcome(term).unwrap();
    for outcome in [direct, materialized] {
        assert_eq!(outcome.status, "Exact");
        assert_eq!(outcome.coverage, "Full");
        assert!(session.project_conditions(outcome.result_id).is_empty());
        assert!(session.project_diagnostics(outcome.result_id).is_empty());
        assert_eq!(session.render_maple_result(outcome.result_id).unwrap(), "1/3");
    }
}

#[test]
fn maple_host_sequence_observes_last_result_and_preserves_prior_result() {
    let session = Session::new();
    let first = session.evaluate_maple_form(&fraction()).unwrap();
    let program = MapleProgram {
        expressions: vec![fraction(), MapleForm::Integer("123456789012345678901234567890".to_owned())],
        span: None,
    };
    let last = session.evaluate_maple_program(&program).unwrap();
    assert_eq!(last.status, "Exact");
    assert_eq!(last.coverage, "Full");
    assert_eq!(session.render_maple_result(last.result_id).unwrap(), "123456789012345678901234567890");
    assert_eq!(session.render_maple_result(first.result_id).unwrap(), "1/3");
    assert_ne!(first.result_id, last.result_id);
}

#[test]
fn maple_host_frontend_failure_does_not_fabricate_athena_code() {
    let session = Session::new();
    let unsupported = MapleForm::Located {
        span: MapleSpan { start: 2, end: 7 },
        expression: Box::new(MapleForm::Decimal("0.125".to_owned())),
    };
    let before = session.math_session.borrow().arena.len();
    let error = session.evaluate_maple_form(&unsupported).unwrap_err();
    assert_eq!(error.athena_code, None);
    assert!(error.message.contains("unsupported"));
    assert!(error.message.contains("2..7"));
    assert_eq!(session.math_session.borrow().arena.len(), before);
    let program = MapleProgram { expressions: vec![fraction(), unsupported], span: None };
    assert!(session.evaluate_maple_program(&program).is_err());
    assert_eq!(session.math_session.borrow().arena.len(), before);
    assert!(session.evaluate_maple_program(&MapleProgram { expressions: vec![], span: None }).is_err());
}

#[test]
fn maple_host_render_is_read_only_and_missing_results_are_errors() {
    let session = Session::new();
    let outcome = session.evaluate_maple_form(&fraction()).unwrap();
    let before = format!("{:?}", session.math_session.borrow().results.get(outcome.result_id).unwrap());
    let terms = session.math_session.borrow().arena.len();
    assert_eq!(session.render_maple_result(outcome.result_id).unwrap(), "1/3");
    assert_eq!(session.math_session.borrow().arena.len(), terms);
    assert_eq!(format!("{:?}", session.math_session.borrow().results.get(outcome.result_id).unwrap()), before);
    let unknown = session.render_maple_result(athena::types::ResultId(u32::MAX)).unwrap_err();
    assert_eq!(unknown.athena_code, None);
    let fresh = Session::new();
    assert!(fresh.render_maple_result(outcome.result_id).is_err());
}

#[test]
fn maple_host_nested_lists_do_not_materialize_matrices() {
    let session = Session::new();
    let form = MapleForm::List(vec![MapleForm::List(vec![MapleForm::Integer("1".to_owned())]), MapleForm::List(vec![])]);
    let term = session.lower_maple_form(&form).unwrap();
    let ms = session.math_session.borrow();
    let Some(athena::ir::TermNode::Collection { kind, elements }) = ms.arena.get(term)
    else {
        panic!("explicit list")
    };
    assert_eq!(*kind, athena::types::CollectionKind::OrderedCollection);
    for child in elements {
        assert!(matches!(
            ms.arena.get(*child),
            Some(athena::ir::TermNode::Collection { kind: athena::types::CollectionKind::OrderedCollection, .. })
        ));
    }
}
