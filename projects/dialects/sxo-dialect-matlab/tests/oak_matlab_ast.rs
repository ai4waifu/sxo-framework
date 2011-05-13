//! Oak `oak-matlab` AST builder tests (migrated from upstream oak.rs).

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_matlab::{
    MatlabBuilder, MatlabLanguage,
    ast::{Expression, MatlabRoot, Statement},
    lexer::token_type::MatlabTokenType,
};

fn build(input: &str) -> MatlabRoot {
    let language = MatlabLanguage::default();
    let builder = MatlabBuilder::new(&language);
    let source = SourceText::new(input.to_string());
    let mut cache = ParseSession::<MatlabLanguage>::default();
    builder.build(&source, &[], &mut cache).result.expect("build ok")
}

#[test]
fn ast_binary_owned() {
    let root = build("a + b");
    assert_eq!(root.items.len(), 1);
    match &root.items[0] {
        Statement::Expr(Expression::Binary(bin)) => {
            assert_eq!(bin.operator, MatlabTokenType::Plus);
            match &bin.lhs {
                Expression::Symbol(id) => assert_eq!(id.name, "a"),
                other => panic!("expected symbol lhs, got {other:?}"),
            }
        }
        other => panic!("expected binary expr, got {other:?}"),
    }
}

#[test]
fn ast_and_infix_owned() {
    let root = build("1 & 0");
    match &root.items[0] {
        Statement::Expr(Expression::Binary(bin)) => {
            assert_eq!(bin.operator, MatlabTokenType::And);
        }
        other => panic!("expected And binary, got {other:?}"),
    }
}

#[test]
fn ast_call_owned() {
    let root = build("sin(x, y)");
    match &root.items[0] {
        Statement::Expr(Expression::Call { head, arguments, .. }) => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "sin"),
                other => panic!("expected sin head, got {other:?}"),
            }
            assert_eq!(arguments.len(), 2);
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn ast_if_owned() {
    let root = build("if 1, 2, else, 3, end");
    match &root.items[0] {
        Statement::If { condition, then_body, else_body, .. } => {
            assert!(matches!(condition, Expression::Literal { .. }));
            assert_eq!(then_body.len(), 1);
            assert_eq!(else_body.len(), 1);
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn ast_array_owned() {
    let root = build("[1, 2; 3]");
    match &root.items[0] {
        Statement::Expr(Expression::Array { rows, .. }) => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[1].len(), 1);
        }
        other => panic!("expected Array, got {other:?}"),
    }
}

#[test]
fn ast_cell_array_owned() {
    let root = build("{1, 2}");
    match &root.items[0] {
        Statement::Expr(Expression::CellArray { rows, .. }) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 2);
        }
        other => panic!("expected CellArray, got {other:?}"),
    }
}

#[test]
fn ast_cell_array_in_call_args() {
    let root = build("cellfun(@numel, {1, 2})");
    match &root.items[0] {
        Statement::Expr(Expression::Call { head, arguments, .. }) => {
            match head.as_ref() {
                Expression::Symbol(id) => assert_eq!(id.name, "cellfun"),
                other => panic!("expected cellfun head, got {other:?}"),
            }
            assert_eq!(arguments.len(), 2, "cell brace must stay one argument, got {arguments:?}");
            assert!(matches!(arguments[0], Expression::FunctionHandle { .. }));
            match &arguments[1] {
                Expression::CellArray { rows, .. } => {
                    assert_eq!(rows.len(), 1);
                    assert_eq!(rows[0].len(), 2);
                }
                other => panic!("expected CellArray arg, got {other:?}"),
            }
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn ast_for_owned() {
    let root = build("for i=1:3, i, end");
    match &root.items[0] {
        Statement::For { header, body, .. } => {
            assert!(matches!(header, Expression::Binary(_)));
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn ast_parfor_owned() {
    let root = build("parfor i=1:3, i, end");
    match &root.items[0] {
        Statement::Parfor { header, body, .. } => {
            assert!(matches!(header, Expression::Binary(_)));
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected Parfor, got {other:?}"),
    }
}

#[test]
fn ast_spmd_owned() {
    let root = build("spmd, 1, end");
    match &root.items[0] {
        Statement::Spmd { body, .. } => {
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected Spmd, got {other:?}"),
    }
}

#[test]
fn ast_try_owned() {
    let root = build("try, 2, catch, 3, end");
    match &root.items[0] {
        Statement::Try { body, catch_name, catch_body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(catch_name.is_none());
            assert_eq!(catch_body.len(), 1);
        }
        other => panic!("expected Try, got {other:?}"),
    }
}

#[test]
fn ast_command_owned() {
    let root = build("hold on");
    assert_eq!(root.items.len(), 1, "command must be one statement, got {root:?}");
    match &root.items[0] {
        Statement::Command { name, args, .. } => {
            assert_eq!(name.name, "hold");
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expression::Symbol(id) => assert_eq!(id.name, "on"),
                other => panic!("expected symbol arg, got {other:?}"),
            }
        }
        other => panic!("expected Command, got {other:?}"),
    }
}

#[test]
fn ast_command_keyword_bareword_args() {
    let root = build("dbstop if error");
    assert_eq!(root.items.len(), 1, "command with keyword bareword must be one statement, got {root:?}");
    let (name, args) = root.primary().expect("primary").as_command().expect("as_command");
    assert_eq!(name.name, "dbstop");
    assert_eq!(args.len(), 2, "got {args:?}");
    match &args[0] {
        Expression::Symbol(id) => assert_eq!(id.name, "if"),
        other => panic!("expected keyword bareword as symbol, got {other:?}"),
    }
    match &args[1] {
        Expression::Symbol(id) => assert_eq!(id.name, "error"),
        other => panic!("expected symbol, got {other:?}"),
    }
}

#[test]
fn ast_command_disp_literal() {
    let root = build("disp 1");
    let (name, args) = root.primary().expect("primary").as_command().expect("as_command");
    assert_eq!(name.name, "disp");
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0], Expression::Literal { .. }));
}

#[test]
fn ast_call_not_command() {
    let root = build("sin(x)");
    assert!(root.primary().and_then(|s| s.as_expr()).and_then(|e| e.as_call()).is_some());
}

#[test]
fn ast_methods_call_keyword_head() {
    let root = build("methods('double')");
    assert_eq!(root.items.len(), 1, "got {root:?}");
    let (head, args) = root.primary().and_then(|s| s.as_expr()).and_then(|e| e.as_call()).expect("methods(...) call");
    match head {
        Expression::Symbol(id) => assert_eq!(id.name, "methods"),
        other => panic!("expected methods symbol head, got {other:?}"),
    }
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0], Expression::Literal { .. }));
}

#[test]
fn ast_assign_not_command() {
    let root = build("x = 1");
    assert!(root.primary().and_then(|s| s.as_expr()).and_then(|e| e.as_assignment()).is_some());
}

#[test]
fn ast_global_owned() {
    let root = build("global x y");
    assert_eq!(root.items.len(), 1, "got {root:?}");
    let names = root.primary().expect("primary").as_global().expect("as_global");
    assert_eq!(names.len(), 2);
    assert_eq!(names[0].name, "x");
    assert_eq!(names[1].name, "y");
}

#[test]
fn ast_persistent_owned() {
    let root = build("persistent z");
    let names = root.primary().expect("primary").as_persistent().expect("as_persistent");
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "z");
}

#[test]
fn ast_member_owned() {
    let root = build("containers.Map");
    assert_eq!(root.items.len(), 1, "got {root:?}");
    let (object, field) = root.primary().expect("primary").as_expr().expect("expr").as_member().expect("as_member");
    assert_eq!(object.as_symbol().map(|s| s.name.as_str()), Some("containers"));
    assert_eq!(field.name, "Map");
}

#[test]
fn ast_member_then_call() {
    let root = build("containers.Map('a', 1)");
    match &root.items[0] {
        Statement::Expr(Expression::Call { head, arguments, .. }) => {
            let (object, field) = head.as_member().expect("member head");
            assert_eq!(object.as_symbol().map(|s| s.name.as_str()), Some("containers"));
            assert_eq!(field.name, "Map");
            assert_eq!(arguments.len(), 2);
        }
        other => panic!("expected Call(Member, …), got {other:?}"),
    }
}

#[test]
fn ast_dot_times_not_member() {
    let root = build("a .* b");
    let bin = root.primary().expect("primary").as_expr().expect("expr").as_elementwise().expect("elementwise");
    assert_eq!(bin.operator, MatlabTokenType::DotTimes);
}

#[test]
fn ast_typed_accessors_call_array_assign_colon() {
    let call_root = build("sin(x, y)");
    let call = call_root.primary().expect("primary").as_expr().expect("expr");
    let (head, args) = call.as_call().expect("as_call");
    assert_eq!(head.as_symbol().map(|s| s.name.as_str()), Some("sin"));
    assert_eq!(args.len(), 2);

    let array_root = build("[1, 2; 3]");
    let array = array_root.primary().expect("primary").as_expr().expect("expr");
    let rows = array.as_array().expect("as_array");
    assert_eq!(rows.len(), 2);

    let assign_root = build("x = 1");
    let assign = assign_root.primary().expect("primary").as_expr().expect("expr");
    let (lhs, rhs) = assign.as_assignment().expect("as_assignment");
    assert_eq!(lhs.as_symbol().map(|s| s.name.as_str()), Some("x"));
    assert_eq!(rhs.as_literal().map(|(t, _)| t), Some("1"));

    let colon_root = build("1:3");
    let colon = colon_root.primary().expect("primary").as_expr().expect("expr");
    let (lhs, rhs) = colon.as_colon().expect("as_colon");
    assert_eq!(lhs.as_literal().map(|(t, _)| t), Some("1"));
    assert_eq!(rhs.as_literal().map(|(t, _)| t), Some("3"));
}

#[test]
fn ast_typed_accessors_statements() {
    let if_root = build("if 1, 2, else, 3, end");
    let view = if_root.primary().expect("primary").as_if().expect("as_if");
    assert!(view.condition.as_literal().is_some());
    assert_eq!(view.then_body.len(), 1);
    assert_eq!(view.else_body.len(), 1);

    let for_root = build("for i=1:3, i, end");
    let (header, body) = for_root.primary().expect("primary").as_for().expect("as_for");
    assert!(header.as_assignment().is_some() || header.as_binary().is_some());
    assert_eq!(body.len(), 1);

    let parfor_root = build("parfor i=1:3, i, end");
    let (header, body) = parfor_root.primary().expect("primary").as_parfor().expect("as_parfor");
    assert!(header.as_assignment().is_some() || header.as_binary().is_some());
    assert_eq!(body.len(), 1);

    let spmd_root = build("spmd, 1, end");
    let body = spmd_root.primary().expect("primary").as_spmd().expect("as_spmd");
    assert_eq!(body.len(), 1);

    let try_root = build("try, 2, catch, 3, end");
    let view = try_root.primary().expect("primary").as_try().expect("as_try");
    assert_eq!(view.body.len(), 1);
    assert!(view.catch_name.is_none());
    assert_eq!(view.catch_body.len(), 1);
}

#[test]
fn ast_typed_accessors_matrix_ops() {
    let ld_root = build("A \\ b");
    let ld = ld_root.primary().expect("primary").as_expr().expect("expr");
    let (a, b) = ld.as_left_divide().expect("as_left_divide");
    assert_eq!(a.as_symbol().map(|s| s.name.as_str()), Some("A"));
    assert_eq!(b.as_symbol().map(|s| s.name.as_str()), Some("b"));

    let ew_root = build("A .* B");
    let ew = ew_root.primary().expect("primary").as_expr().expect("expr");
    let bin = ew.as_elementwise().expect("as_elementwise");
    assert_eq!(bin.operator, MatlabTokenType::DotTimes);

    let power_root = build("A .^ 2");
    let power = power_root.primary().expect("primary").as_expr().expect("expr");
    assert_eq!(power.as_elementwise().map(|b| b.operator), Some(MatlabTokenType::DotPower));

    let neg_pow_root = build("-x^2");
    let neg_pow = neg_pow_root.primary().expect("primary").as_expr().expect("expr");
    let prefix = neg_pow.as_prefix().expect("unary minus wraps power");
    assert_eq!(prefix.operator, MatlabTokenType::Minus);
    assert!(prefix.operand.as_binary().is_some(), "operand should be x^2 BinaryExpr");
    assert_eq!(prefix.operand.as_binary().map(|b| b.operator), Some(MatlabTokenType::Power));

    let tr_root = build("A'");
    let tr = tr_root.primary().expect("primary").as_expr().expect("expr");
    let u = tr.as_transpose().expect("as_transpose");
    assert_eq!(u.operator, MatlabTokenType::Transpose);
}

#[test]
fn ast_function_and_return_owned() {
    let root = build("function out = f(x)\n    out = x;\n    return;\nend");
    match &root.items[0] {
        Statement::Function { header, body, .. } => {
            match header {
                Expression::Binary(bin) => assert_eq!(bin.operator, MatlabTokenType::Assign),
                other => panic!("expected assign header, got {other:?}"),
            }
            assert!(body.iter().any(|s| matches!(s, Statement::Return { value: None, .. })));
        }
        other => panic!("expected Function, got {other:?}"),
    }
}
