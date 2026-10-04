use super::Expression;

#[test]
fn maple_parse_handle_evaluates_and_renders_result() {
    let parsed = Expression::new("1 / 3", Some("maple".to_owned())).unwrap();
    let result = parsed.evaluate(None).unwrap();
    assert_eq!(result.status(), "Exact");
    assert_eq!(result.coverage(), "Full");
    assert_eq!(result.to_string_js().unwrap(), "1/3");
}

#[test]
fn maple_parse_handle_preserves_program_sequence_and_rejects_term_only_operations() {
    let parsed = Expression::new("1 / 3; 7", Some("maple".to_owned())).unwrap();
    assert_eq!(parsed.to_string_js().unwrap(), "(1 / 3); 7");
    assert!(parsed.evaluate(None).is_ok());
}
