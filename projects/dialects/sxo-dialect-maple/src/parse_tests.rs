use super::{form_from_ast, parse_maple_form, parse_maple_program, program_from_ast};
use crate::{
    MapleError, MapleForm, MapleOperator, MapleProgram, MapleSpan, lower_program_request, lower_request,
    render_maple_result_projection,
};
use athena::{AthenaEngine, Session, numeric::number_to_wire, runtime::values::arena::number_from_id};
use oak_maple::ast::{MapleExpression, MapleExpressionKind, MapleOperator as OakOperator, MapleRoot, MapleSpan as OakSpan};

fn ast_integer() -> MapleExpression {
    MapleExpression { kind: MapleExpressionKind::Integer("0007".to_owned()), span: OakSpan { start: 2, end: 6 } }
}

#[test]
fn typed_ast_binary_operators_preserve_independent_form_identity() {
    for (source_operator, expected_operator) in [
        (OakOperator::Add, MapleOperator::Add),
        (OakOperator::Subtract, MapleOperator::Subtract),
        (OakOperator::Multiply, MapleOperator::Multiply),
        (OakOperator::Divide, MapleOperator::Divide),
        (OakOperator::Power, MapleOperator::Power),
    ] {
        let expression = MapleExpression {
            kind: MapleExpressionKind::Binary {
                operator: source_operator,
                left: Box::new(ast_integer()),
                right: Box::new(ast_integer()),
            },
            span: OakSpan { start: 0, end: 8 },
        };
        let form = form_from_ast(&expression).unwrap();
        let expected_operand = MapleForm::Located {
            span: MapleSpan { start: 2, end: 6 },
            expression: Box::new(MapleForm::Integer("0007".to_owned())),
        };
        assert_eq!(
            form,
            MapleForm::Located {
                span: MapleSpan { start: 0, end: 8 },
                expression: Box::new(MapleForm::Binary {
                    operator: expected_operator,
                    left: Box::new(expected_operand.clone()),
                    right: Box::new(expected_operand),
                }),
            }
        );
    }
}

#[test]
fn typed_ast_invalid_unary_operators_are_syntax_errors() {
    for operator in [OakOperator::Multiply, OakOperator::Divide, OakOperator::Power] {
        let expression = MapleExpression {
            kind: MapleExpressionKind::Unary { operator, operand: Box::new(ast_integer()) },
            span: OakSpan { start: 1, end: 6 },
        };
        let MapleError::Syntax(diagnostics) = form_from_ast(&expression).unwrap_err()
        else {
            panic!("invalid unary operator must be rejected")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].offset, Some(1));
    }
}

#[test]
fn typed_ast_invalid_ranges_are_rejected_at_every_boundary() {
    let mut invalid_ranges = vec![OakSpan { start: 8, end: 2 }];
    if usize::BITS > u32::BITS {
        invalid_ranges.push(OakSpan { start: 0, end: u32::MAX as usize + 1 });
    }
    for span in invalid_ranges {
        let expected = MapleError::InvalidSpan(MapleSpan { start: span.start, end: span.end });
        let mut child = ast_integer();
        child.span = span.clone();
        assert_eq!(form_from_ast(&child), Err(expected.clone()));
        let parent = MapleExpression { kind: MapleExpressionKind::List(vec![child]), span: OakSpan { start: 0, end: 10 } };
        assert_eq!(form_from_ast(&parent), Err(expected.clone()));
        let root = MapleRoot { expressions: vec![ast_integer()], span };
        assert_eq!(program_from_ast(&root), Err(expected));
    }
}

#[test]
fn converted_program_is_owned_and_does_not_normalize_integer_spelling() {
    let mut root = MapleRoot { expressions: vec![ast_integer()], span: OakSpan { start: 0, end: 8 } };
    let program = program_from_ast(&root).unwrap();
    root.expressions[0].kind = MapleExpressionKind::Symbol("changed".to_owned());
    root.expressions.clear();
    drop(root);
    assert_eq!(program.span, Some(MapleSpan { start: 0, end: 8 }));
    assert_eq!(
        program.expressions,
        vec![MapleForm::Located {
            span: MapleSpan { start: 2, end: 6 },
            expression: Box::new(MapleForm::Integer("0007".to_owned())),
        }]
    );
    assert_eq!(execute_program(&program), "7");
}

fn execute_program(program: &MapleProgram) -> String {
    let mut session = Session::new();
    let request = lower_program_request(&mut session, program).expect("program lowering");
    let result_id = AthenaEngine::new().execute_request(&mut session, request).expect("execution");
    let result = session.results.get(result_id).expect("owned result");
    assert_eq!(result.status.name(), "Exact");
    assert_eq!(result.coverage.name(), "Full");
    assert!(result.conditions.is_empty());
    assert!(result.diagnostics.is_empty());
    let root = result.symbolic_term.expect("numeric result projection");
    let expected = number_to_wire(number_from_id(&session, root).expect("number")).to_render_string();
    let rendered = render_maple_result_projection(&session, result_id).expect("Maple result display");
    assert_eq!(rendered, expected);
    rendered
}

#[test]
fn source_scalar_chain_executes_only_through_athena() {
    for (input, expected) in [
        ("1 + 2 * 3", "7"),
        ("1 / 3", "1/3"),
        ("123456789012345678901234567890", "123456789012345678901234567890"),
        ("-2^2", "-4"),
        ("(-2)^2", "4"),
        ("8 / 4 / 2", "1"),
        ("2^(3^2)", "512"),
        ("(2^3)^2", "64"),
    ] {
        assert_eq!(execute_program(&parse_maple_program(input).unwrap()), expected, "{input}");
    }
}

#[test]
fn parsed_request_matches_structured_direct_request() {
    let direct = MapleProgram {
        expressions: vec![MapleForm::Binary {
            operator: MapleOperator::Divide,
            left: Box::new(MapleForm::Integer("1".to_owned())),
            right: Box::new(MapleForm::Integer("3".to_owned())),
        }],
        span: None,
    };
    assert_eq!(execute_program(&parse_maple_program("1/3").unwrap()), execute_program(&direct));
}

#[test]
fn unicode_comments_and_leading_trivia_do_not_shift_source_ranges() {
    let input = "  # 中文
  123 + 7;";
    let form = parse_maple_form(input).unwrap();
    let MapleForm::Located { span, expression } = form
    else {
        panic!("located expression")
    };
    assert_eq!(&input[span.start..span.end], "123 + 7");
    let MapleForm::Binary { left, .. } = expression.as_ref()
    else {
        panic!("binary")
    };
    let MapleForm::Located { span, .. } = left.as_ref()
    else {
        panic!("located integer")
    };
    assert_eq!(&input[span.start..span.end], "123");
    assert_eq!(span.start, input.find("123").unwrap());
}

#[test]
fn invalid_syntax_never_returns_a_partial_executable_form() {
    for input in ["1+", "f(1,)", "[1,2", "1 2", "2^3^2", "x:=1", "if x then y end if", "1; 2+", "😀"] {
        let error = parse_maple_program(input).unwrap_err();
        let MapleError::Syntax(diagnostics) = error
        else {
            panic!("expected syntax error: {input}")
        };
        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.offset.is_some()), "{input}");
    }
}

#[test]
fn parsed_form_owns_literal_spelling_after_source_drop() {
    let form = {
        let input = String::from("000123");
        parse_maple_form(&input).unwrap()
    };
    let MapleForm::Located { expression, .. } = form
    else {
        panic!("located")
    };
    assert_eq!(*expression, MapleForm::Integer("000123".to_owned()));
}

#[test]
fn syntax_supported_decimal_is_execution_unsupported_with_exact_source_range() {
    let form = parse_maple_form(" 0.125 ").unwrap();
    let mut session = Session::new();
    let before = session.arena.len();
    let error = lower_request(&mut session, &form).unwrap_err();
    assert!(
        matches!(error, MapleError::Located { span: MapleSpan { start: 1, end: 6 }, error } if matches!(*error, MapleError::Unsupported(_)))
    );
    assert_eq!(session.arena.len(), before);
}

#[test]
fn named_call_is_preserved_and_never_string_dispatched() {
    let form = parse_maple_form("solve(x)").unwrap();
    let MapleForm::Located { expression, .. } = &form
    else {
        panic!("located")
    };
    assert!(matches!(expression.as_ref(), MapleForm::Call { name, arguments } if name == "solve" && arguments.len() == 1));
    let mut session = Session::new();
    assert!(
        matches!(lower_request(&mut session, &form), Err(MapleError::Located { error, .. }) if matches!(*error, MapleError::Unsupported(_)))
    );
}

#[test]
fn empty_root_and_multiple_expressions_are_not_silently_collapsed() {
    let empty = parse_maple_program(" # comment").unwrap();
    assert!(empty.expressions.is_empty());
    assert!(matches!(parse_maple_form(" # comment"), Err(MapleError::EmptyProgram)));
    let program = parse_maple_program("1;2+3;").unwrap();
    assert_eq!(program.expressions.len(), 2);
    assert_eq!(execute_program(&program), "5");
    assert!(matches!(parse_maple_form("1;2"), Err(MapleError::Syntax(_))));
}

#[test]
fn parsed_nested_lists_remain_owned_lists() {
    let form = parse_maple_form("[[1,2],[3,4]]").unwrap();
    let mut session = Session::new();
    let athena::api::AthenaRequest::Term(term) = lower_request(&mut session, &form).unwrap()
    else {
        panic!("term")
    };
    let Some(athena::ir::TermNode::Collection { kind, elements }) = session.arena.get(term)
    else {
        panic!("list")
    };
    assert_eq!(*kind, athena::types::CollectionKind::OrderedCollection);
    assert_eq!(elements.len(), 2);
}
