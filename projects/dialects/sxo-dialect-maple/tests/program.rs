//! Ordered Maple source programs lower into neutral Athena control flow.

use athena::{
    AthenaEngine, Session,
    api::{AthenaRequest, ControlPlan},
    numeric::number_to_wire,
    runtime::values::arena::number_from_id,
};
use sxo_dialect_maple::{MapleError, MapleForm, MapleProgram, lower_program_request};

fn program(expressions: Vec<MapleForm>) -> MapleProgram {
    MapleProgram { expressions, span: None }
}

#[test]
fn one_expression_preserves_the_scalar_request_boundary() {
    let mut session = Session::new();
    let request = lower_program_request(&mut session, &program(vec![MapleForm::Integer("7".to_owned())])).unwrap();
    assert!(matches!(request, AthenaRequest::Term(_)));
}

#[test]
fn ordered_sequence_uses_athena_and_observes_last_result() {
    let mut session = Session::new();
    let form = program(vec![MapleForm::Integer("7".to_owned()), MapleForm::Integer("11".to_owned())]);
    let request = lower_program_request(&mut session, &form).unwrap();
    assert!(matches!(&request, AthenaRequest::Control(ControlPlan::Sequence { steps }) if steps.len() == 2));
    let result_id = AthenaEngine::new().execute_request(&mut session, request).unwrap();
    let result = session.results.get(result_id).expect("owned sequence result");
    assert_eq!(result.status.name(), "Exact");
    assert_eq!(result.coverage.name(), "Full");
    assert!(result.conditions.is_empty());
    assert!(result.diagnostics.is_empty());
    let root = result.symbolic_term.expect("last result projection");
    assert_eq!(number_to_wire(number_from_id(&session, root).unwrap()).to_render_string(), "11");
}

#[test]
fn invalid_later_expression_cannot_allocate_an_earlier_expression() {
    let mut session = Session::new();
    let form = program(vec![MapleForm::Integer("7".to_owned()), MapleForm::Decimal("0.1".to_owned())]);
    let before = session.arena.len();
    assert!(matches!(lower_program_request(&mut session, &form), Err(MapleError::Unsupported(_))));
    assert_eq!(session.arena.len(), before);
}

#[test]
fn empty_syntax_root_is_not_an_executable_null() {
    let mut session = Session::new();
    let before = session.arena.len();
    assert_eq!(lower_program_request(&mut session, &program(vec![])).unwrap_err(), MapleError::EmptyProgram);
    assert_eq!(session.arena.len(), before);
}
