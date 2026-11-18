//! Source provenance and failure atomicity for Maple lowering.

use science::{Session, api::CasRequest, ir::TermNode, types::SourceSpan};
use sxo_dialect_maple::{MapleError, MapleForm, MapleOperator, MapleSpan, lower_request, render_maple_form};

fn located(span: MapleSpan, expression: MapleForm) -> MapleForm {
    MapleForm::Located { span, expression: Box::new(expression) }
}

#[test]
fn distinct_source_ranges_reach_the_neutral_term_boundary() {
    let mut session = Session::new();
    let form = located(
        MapleSpan { start: 4, end: 11 },
        MapleForm::Binary {
            operator: MapleOperator::Add,
            left: Box::new(located(MapleSpan { start: 4, end: 6 }, MapleForm::Integer("11".to_owned()))),
            right: Box::new(located(MapleSpan { start: 9, end: 11 }, MapleForm::Integer("23".to_owned()))),
        },
    );
    let CasRequest::Term(root) = lower_request(&mut session, &form).unwrap()
    else {
        panic!("term request")
    };
    assert_eq!(session.arena.span(root), Some(SourceSpan { start: 4, end: 11 }));
    let Some(TermNode::Application { arguments, .. }) = session.arena.get(root)
    else {
        panic!("addition")
    };
    assert_eq!(session.arena.span(arguments[0]), Some(SourceSpan { start: 4, end: 6 }));
    assert_eq!(session.arena.span(arguments[1]), Some(SourceSpan { start: 9, end: 11 }));
    assert_eq!(render_maple_form(&form), "(11 + 23)");
}

#[test]
fn nearest_source_construct_owns_unsupported_diagnostic() {
    let mut session = Session::new();
    let inner_span = MapleSpan { start: 6, end: 9 };
    let form = located(
        MapleSpan { start: 0, end: 10 },
        MapleForm::List(vec![MapleForm::Integer("123".to_owned()), located(inner_span, MapleForm::Decimal("0.1".to_owned()))]),
    );
    let before = session.arena.len();
    let error = lower_request(&mut session, &form).unwrap_err();
    assert_eq!(session.arena.len(), before);
    assert!(
        matches!(&error, MapleError::Located { span, error } if *span == inner_span && matches!(error.as_ref(), MapleError::Unsupported(_)))
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn invalid_spans_never_truncate_into_athena_offsets() {
    let mut session = Session::new();
    let mut spans = vec![MapleSpan { start: 9, end: 2 }];
    if usize::BITS > u32::BITS {
        spans.push(MapleSpan { start: 0, end: u32::MAX as usize + 1 });
    }
    for span in spans {
        let form = located(span, MapleForm::Integer("7".to_owned()));
        let before = session.arena.len();
        assert!(matches!(lower_request(&mut session, &form), Err(MapleError::InvalidSpan(actual)) if actual == span));
        assert_eq!(session.arena.len(), before);
    }
}

#[test]
fn canonical_terms_do_not_replace_source_occurrence_spans() {
    let mut session = Session::new();
    let first = located(MapleSpan { start: 0, end: 2 }, MapleForm::Integer("11".to_owned()));
    let second = located(MapleSpan { start: 10, end: 12 }, MapleForm::Integer("11".to_owned()));
    let CasRequest::Term(first_id) = lower_request(&mut session, &first).unwrap()
    else {
        panic!("first term")
    };
    let CasRequest::Term(second_id) = lower_request(&mut session, &second).unwrap()
    else {
        panic!("second term")
    };
    assert_eq!(first_id, second_id);
    assert_eq!(session.arena.span(second_id), Some(SourceSpan { start: 0, end: 2 }));
    assert!(matches!(second, MapleForm::Located { span: MapleSpan { start: 10, end: 12 }, .. }));
}

#[test]
fn whole_form_validation_precedes_any_math_allocation() {
    let mut session = Session::new();
    let form = MapleForm::List(vec![
        MapleForm::Integer("12345678901234567890".to_owned()),
        MapleForm::Symbol("valid_name".to_owned()),
        MapleForm::Integer("1e3".to_owned()),
    ]);
    let before = session.arena.len();
    assert!(matches!(lower_request(&mut session, &form), Err(MapleError::InvalidInteger(_))));
    assert_eq!(session.arena.len(), before);
}
