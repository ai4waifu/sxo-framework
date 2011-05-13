//! Oak `oak-wolfram` AST builder tests (migrated from upstream oak.rs).

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_wolfram::{
    WolframBuilder, WolframLanguage,
    ast::{AssignmentTiming, Expression, WolframRoot},
    lexer::token_type::WolframTokenType,
};

fn build(input: &str) -> WolframRoot {
    let language = WolframLanguage::default();
    let builder = WolframBuilder::new(&language);
    let source = SourceText::new(input.to_string());
    let mut cache = ParseSession::<WolframLanguage>::default();
    builder.build(&source, &[], &mut cache).result.expect("build ok")
}

#[test]
fn ast_binary_owned() {
    let root = build("a + b");
    assert_eq!(root.expressions.len(), 1);
    match &root.expressions[0] {
        Expression::Binary(bin) => {
            assert_eq!(bin.operator, WolframTokenType::Plus);
            match &bin.lhs {
                Expression::Symbol(id) => assert_eq!(id.name, "a"),
                other => panic!("expected symbol lhs, got {other:?}"),
            }
            match &bin.rhs {
                Expression::Symbol(id) => assert_eq!(id.name, "b"),
                other => panic!("expected symbol rhs, got {other:?}"),
            }
        }
        other => panic!("expected Binary, got {other:?}"),
    }
}

#[test]
fn ast_call_owned() {
    let root = build("If[1, 2, 3]");
    match &root.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "If"),
                other => panic!("expected If head, got {other:?}"),
            }
            assert_eq!(arguments.len(), 3);
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn ast_part_owned() {
    let root = build("{1, 2}[[1]]");
    match &root.expressions[0] {
        Expression::Part { expression, indices, .. } => {
            assert!(matches!(expression.as_ref(), Expression::List { .. }));
            assert_eq!(indices.len(), 1);
        }
        other => panic!("expected Part, got {other:?}"),
    }
}

#[test]
fn ast_pattern_owned() {
    let root = build("x_");
    match &root.expressions[0] {
        Expression::Pattern { name, blank, .. } => {
            match name.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "x"),
                other => panic!("expected symbol name, got {other:?}"),
            }
            assert_eq!(*blank, WolframTokenType::Underscore);
        }
        other => panic!("expected Pattern, got {other:?}"),
    }
}

#[test]
fn ast_compound_semicolon_owned() {
    let root = build("1; 2");
    assert_eq!(root.expressions.len(), 1);
    match &root.expressions[0] {
        Expression::Binary(bin) => {
            assert_eq!(bin.operator, WolframTokenType::Semicolon);
        }
        other => panic!("expected Binary semicolon, got {other:?}"),
    }
}

#[test]
fn ast_full_form_binary_and_part() {
    let root = build("a + b").full_form();
    match &root.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "Plus"),
                other => panic!("expected Plus, got {other:?}"),
            }
            assert_eq!(arguments.len(), 2);
        }
        other => panic!("expected Call Plus, got {other:?}"),
    }

    let part = build("{1}[[1]]").full_form();
    match &part.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "Part"),
                other => panic!("expected Part, got {other:?}"),
            }
            assert_eq!(arguments.len(), 2);
            assert!(matches!(arguments[0], Expression::List { .. }));
        }
        other => panic!("expected Call Part, got {other:?}"),
    }
}

#[test]
fn ast_unary_minus_power_precedence() {
    // `-x^2` must be Times[-1, Power[x, 2]], not Power[Times[-1, x], 2].
    let ff = build("-x^2").full_form();
    match &ff.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Times"));
            assert_eq!(arguments.len(), 2);
            match &arguments[1] {
                Expression::Call { head, arguments: pow_args, .. } => {
                    assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Power"));
                    assert_eq!(pow_args.len(), 2);
                }
                other => panic!("expected Power rhs, got {other:?}"),
            }
        }
        other => panic!("expected Times Call, got {other:?}"),
    }

    let paren = build("(-x)^2").full_form();
    match &paren.expressions[0] {
        Expression::Call { head, .. } => {
            assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Power"));
        }
        other => panic!("expected Power, got {other:?}"),
    }

    let exp = build("Exp[-x^2]").full_form();
    match &exp.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Exp"));
            match &arguments[0] {
                Expression::Call { head, arguments: targs, .. } => {
                    assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Times"));
                    assert!(matches!(&targs[1], Expression::Call { head, .. } if matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Power")));
                }
                other => panic!("expected Times inside Exp, got {other:?}"),
            }
        }
        other => panic!("expected Exp Call, got {other:?}"),
    }
}

#[test]
fn ast_implicit_times_and_d_arity() {
    let times = build("x y").full_form();
    match &times.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "Times"));
            assert_eq!(arguments.len(), 2);
        }
        other => panic!("expected Times, got {other:?}"),
    }

    let d = build("D[x y, x]");
    match &d.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            assert!(matches!(head.as_ref(), Expression::Symbol(id) if id.name == "D"));
            assert_eq!(arguments.len(), 2, "D[x y, x] must keep two args, got {arguments:?}");
            match &arguments[0] {
                Expression::Binary(bin) => assert_eq!(bin.operator, WolframTokenType::Times),
                other => panic!("expected Times binary first arg, got {other:?}"),
            }
        }
        other => panic!("expected D Call, got {other:?}"),
    }
}

#[test]
fn ast_full_form_pattern_and_if_call() {
    let root = build("x_").full_form();
    match &root.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "Pattern"),
                other => panic!("expected Pattern, got {other:?}"),
            }
            assert_eq!(arguments.len(), 2);
            match &arguments[1] {
                Expression::Call { head, arguments: blank_args, .. } => {
                    match head.as_ref() {
                        Expression::Symbol(id) => assert_eq!(id.name, "Blank"),
                        other => panic!("expected Blank, got {other:?}"),
                    }
                    assert!(blank_args.is_empty());
                }
                other => panic!("expected Blank call, got {other:?}"),
            }
        }
        other => panic!("expected Pattern call, got {other:?}"),
    }

    // Control flow stays a Call with symbol head — full_form does not invent IfStmt.
    let iff = build("If[1, 2]").full_form();
    match &iff.expressions[0] {
        Expression::Call { head, arguments, .. } => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "If"),
                other => panic!("expected If, got {other:?}"),
            }
            assert_eq!(arguments.len(), 2);
        }
        other => panic!("expected If call, got {other:?}"),
    }
}

#[test]
fn ast_typed_accessors_call_part_pattern() {
    let call_root = build("If[1, 2, 3]");
    let call = call_root.primary().expect("primary");
    let (head, args) = call.as_call().expect("as_call");
    assert_eq!(head.as_symbol().map(|s| s.name.as_str()), Some("If"));
    assert_eq!(args.len(), 3);
    assert_eq!(args[0].as_literal().map(|(t, _)| t), Some("1"));

    let part_root = build("{1, 2}[[1]]");
    let part = part_root.primary().expect("primary");
    let (expr, indices) = part.as_part().expect("as_part");
    assert!(expr.as_list().is_some());
    assert_eq!(indices.len(), 1);

    let pat_root = build("x_");
    let pat = pat_root.primary().expect("primary");
    let (name, blank) = pat.as_pattern().expect("as_pattern");
    assert_eq!(name.as_symbol().map(|s| s.name.as_str()), Some("x"));
    assert_eq!(blank, WolframTokenType::Underscore);
}

#[test]
fn ast_typed_accessors_assignment_rule_compound() {
    let set_root = build("x = 1");
    let set = set_root.primary().expect("primary");
    let assign = set.as_assignment().expect("as_assignment");
    assert_eq!(assign.timing, AssignmentTiming::Immediate);
    assert_eq!(assign.lhs.as_symbol().map(|s| s.name.as_str()), Some("x"));
    assert_eq!(assign.rhs.as_literal().map(|(t, _)| t), Some("1"));

    let delayed_root = build("x := 1 + 1");
    let delayed = delayed_root.primary().expect("primary");
    let assign = delayed.as_assignment().expect("deferred assignment");
    assert_eq!(assign.timing, AssignmentTiming::Deferred);

    let rule_root = build("a -> b");
    let rule = rule_root.primary().expect("primary");
    let view = rule.as_rule().expect("as_rule");
    assert!(!view.delayed);
    assert_eq!(view.lhs.as_symbol().map(|s| s.name.as_str()), Some("a"));

    let delayed_rule_root = build("a :> b");
    let delayed_rule = delayed_rule_root.primary().expect("primary");
    let view = delayed_rule.as_rule().expect("delayed rule");
    assert!(view.delayed);

    let compound_root = build("1; 2");
    let compound = compound_root.primary().expect("primary");
    let (lhs, rhs) = compound.as_compound().expect("as_compound");
    assert_eq!(lhs.as_literal().map(|(t, _)| t), Some("1"));
    assert_eq!(rhs.as_literal().map(|(t, _)| t), Some("2"));
}
