use super::Expression;

#[test]
fn maple_parse_handle_evaluates_and_renders_result() {
    let parsed = Expression::parse("1 / 3".to_owned(), Some("maple".to_owned())).unwrap();
    assert_eq!(parsed.dialect(), "maple");
    assert_eq!(parsed.to_string_js().unwrap(), "(1 / 3)");
    let result = parsed.evaluate(None).unwrap();
    assert_eq!(result.status(), "Exact");
    assert_eq!(result.coverage(), "Full");
    assert_eq!(result.to_string_js().unwrap(), "1/3");
}

#[test]
fn maple_parse_handle_preserves_program_sequence_and_rejects_term_only_operations() {
    let parsed = Expression::parse("1 / 3; 7".to_owned(), Some("maple".to_owned())).unwrap();
    assert_eq!(parsed.to_string_js().unwrap(), "(1 / 3); 7");
    assert!(parsed.evaluate(None).is_ok());
    assert!(Expression::parse("7".to_owned(), Some("maple".to_owned())).unwrap().to_wolfram().is_err());
    assert!(parsed.to_matlab().is_err());
    assert!(parsed.d("x".to_owned()).is_err());
    assert!(parsed.simplify().is_err());
}

#[test]
fn maple_unknown_dialect_and_invalid_source_are_rejected_before_handle_creation() {
    assert!(Expression::parse("1".to_owned(), Some("unknown".to_owned())).is_err());
    assert!(Expression::parse("1 +".to_owned(), Some("maple".to_owned())).is_err());
}
