//! Direct Maple Form to Athena execution boundary tests.

use science::{CasEngine, Session, api::CasRequest, runtime::values::arena::number_from_id};
use sxo_dialect_maple::{MapleError, MapleForm, MapleOperator, lower_request, render_maple_form};

fn integer(text: &str) -> MapleForm {
    MapleForm::Integer(text.to_owned())
}

fn binary(operator: MapleOperator, left: MapleForm, right: MapleForm) -> MapleForm {
    MapleForm::Binary { operator, left: Box::new(left), right: Box::new(right) }
}

fn execute_number(form: &MapleForm) -> String {
    let mut session = Session::new();
    let request = lower_request(&mut session, form).expect("valid lowering");
    let result_id = CasEngine::new().execute_request(&mut session, request).expect("Athena execution");
    let result = session.results.get(result_id).expect("owned result");
    assert_eq!(result.status.name(), "Exact");
    assert_eq!(result.coverage.name(), "Full");
    assert!(result.conditions.is_empty());
    assert!(result.diagnostics.is_empty());
    let term = result.symbolic_term.expect("result-owned numeric projection");
    science::numeric::number_to_wire(number_from_id(&session, term).expect("numeric result")).to_render_string()
}

#[test]
fn direct_arithmetic_is_executed_by_athena() {
    for (operator, expected) in [
        (MapleOperator::Add, "10"),
        (MapleOperator::Subtract, "6"),
        (MapleOperator::Multiply, "16"),
        (MapleOperator::Divide, "4"),
        (MapleOperator::Power, "64"),
    ] {
        assert_eq!(execute_number(&binary(operator, integer("8"), integer("2"))), expected);
    }
}

#[test]
fn exact_rational_does_not_round_through_machine_float() {
    assert_eq!(execute_number(&binary(MapleOperator::Divide, integer("1"), integer("3"))), "1/3");
}

#[test]
fn exact_large_integer_does_not_round_through_machine_float() {
    let spelling = "123456789012345678901234567890123456789";
    assert_eq!(execute_number(&integer(spelling)), spelling);
}

#[test]
fn unary_signs_and_grouping_preserve_the_form() {
    let form = MapleForm::Negative(Box::new(MapleForm::Grouped(Box::new(MapleForm::Positive(Box::new(integer("7")))))));
    assert_eq!(execute_number(&form), "-7");
    assert_eq!(render_maple_form(&form), "-((+(7)))");
}

#[test]
fn unknown_calls_and_decimal_policy_are_explicitly_unsupported() {
    let mut session = Session::new();
    for form in
        [MapleForm::Decimal("0.1".to_owned()), MapleForm::Call { name: "solve".to_owned(), arguments: vec![integer("1")] }]
    {
        assert!(matches!(lower_request(&mut session, &form), Err(MapleError::Unsupported(_))));
    }
}

#[test]
fn malformed_direct_literals_and_reserved_names_are_rejected() {
    let mut session = Session::new();
    for spelling in ["", "-1", "1.5", "1/2", "1e3", " 1"] {
        assert!(matches!(lower_request(&mut session, &integer(spelling)), Err(MapleError::InvalidInteger(_))));
    }
    for name in ["", "1x", "x y", "if", "proc", "中文"] {
        assert!(matches!(lower_request(&mut session, &MapleForm::Symbol(name.to_owned())), Err(MapleError::InvalidName(_))));
    }
}

#[test]
fn lists_lower_as_lists_not_inferred_matrices() {
    let mut session = Session::new();
    let form = MapleForm::List(vec![MapleForm::List(vec![integer("1"), integer("2")])]);
    let CasRequest::Term(term) = lower_request(&mut session, &form).unwrap()
    else {
        panic!("term request")
    };
    let Some(science::ir::TermNode::Collection { kind, elements }) = session.arena.get(term)
    else {
        panic!("outer list")
    };
    assert_eq!(*kind, science::types::CollectionKind::OrderedCollection);
    assert_eq!(elements.len(), 1);
    assert!(matches!(
        session.arena.get(elements[0]),
        Some(science::ir::TermNode::Collection { kind: science::types::CollectionKind::OrderedCollection, .. })
    ));
}
