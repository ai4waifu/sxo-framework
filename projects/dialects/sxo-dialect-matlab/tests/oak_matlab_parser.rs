//! Oak `oak-matlab` parser conformance tests (migrated from upstream oak.rs).

use oak_core::{Parser, source::SourceText};
use oak_matlab::{MatlabLanguage, MatlabParser, parser::element_type::MatlabElementType};

fn count_kind(node: &oak_core::tree::GreenNode<'_, oak_matlab::MatlabLanguage>, kind: MatlabElementType) -> usize {
    use oak_core::tree::GreenTree;
    let mut n = if node.kind == kind { 1 } else { 0 };
    for child in node.children {
        if let GreenTree::Node(c) = child {
            n += count_kind(c, kind);
        }
    }
    n
}

fn assert_has_kind(input: &str, kind: MatlabElementType) {
    let source = SourceText::new(input.to_string());
    let language = MatlabLanguage::default();
    let mut cache = oak_core::ParseSession::<MatlabLanguage>::default();
    let parser = MatlabParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    let root = output.result.expect(&format!("parse failed for `{input}`"));
    assert!(count_kind(root, kind) >= 1, "expected {kind:?} in `{input}`, got {root:?}");
}

fn parse_ok(input: &str) {
    let source = SourceText::new(input.to_string());
    let language = MatlabLanguage::default();
    let mut cache = oak_core::ParseSession::<MatlabLanguage>::default();
    let parser = MatlabParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    assert!(output.result.is_ok(), "parse failed for `{input}`: {:?}", output.result.err());
}

#[test]
fn test_parser_binary_expr() {
    parse_ok("1+2*3");
}

#[test]
fn test_parser_and_or_infix() {
    assert_has_kind("1 & 0", MatlabElementType::BinaryExpr);
    assert_has_kind("1 | 0", MatlabElementType::BinaryExpr);
    assert_has_kind("true & false", MatlabElementType::BinaryExpr);
}

#[test]
fn test_parser_call() {
    parse_ok("f(x,y)");
}

#[test]
fn test_parser_array() {
    parse_ok("[1,2;3,4]");
}

#[test]
fn test_parser_prefix_postfix() {
    parse_ok("-a'");
    assert_has_kind("A'", MatlabElementType::PostfixExpr);
    assert_has_kind("-a'", MatlabElementType::PostfixExpr);
}

#[test]
fn test_parser_unary_minus_below_power() {
    // MATLAB: `-x^2` is `-(x^2)`, not `(-x)^2`.
    parse_ok("-x^2");
    assert_has_kind("-x^2", MatlabElementType::PrefixExpr);
    assert_has_kind("-x^2", MatlabElementType::BinaryExpr);
}

#[test]
fn test_parser_grouped() {
    parse_ok("(a+b)*c");
}

#[test]
fn test_parser_array_index() {
    assert_has_kind("[1, 2, 3](1:2)", MatlabElementType::Call);
}

#[test]
fn test_parser_if_else_end() {
    assert_has_kind("if 1, 2, else, 3, end", MatlabElementType::IfStmt);
}

#[test]
fn test_parser_while_end() {
    assert_has_kind("while 0, 1, end", MatlabElementType::WhileStmt);
}

#[test]
fn test_parser_for_end() {
    assert_has_kind("for i=1:3, i, end", MatlabElementType::ForStmt);
}

#[test]
fn test_parser_parfor_end() {
    assert_has_kind("parfor i=1:3, i, end", MatlabElementType::ParforStmt);
}

#[test]
fn test_parser_spmd_end() {
    assert_has_kind("spmd, 1, end", MatlabElementType::SpmdStmt);
}

#[test]
fn test_parser_try_catch_end() {
    assert_has_kind("try, 2, catch, 3, end", MatlabElementType::TryStmt);
    assert_has_kind("try, error('e'), catch, 1, end", MatlabElementType::TryStmt);
}

#[test]
fn test_parser_command_stmt() {
    assert_has_kind("hold on", MatlabElementType::CommandStmt);
    assert_has_kind("grid minor", MatlabElementType::CommandStmt);
    assert_has_kind("disp 1", MatlabElementType::CommandStmt);
    assert_has_kind("close all;", MatlabElementType::CommandStmt);
    assert_has_kind("dbstop if error", MatlabElementType::CommandStmt);
}

#[test]
fn test_parser_call_not_command() {
    assert_has_kind("sin(x)", MatlabElementType::Call);
    assert_has_kind("methods('double')", MatlabElementType::Call);
    assert_has_kind("properties(obj)", MatlabElementType::Call);
}

#[test]
fn test_parser_global_persistent_stmt() {
    assert_has_kind("global x", MatlabElementType::GlobalStmt);
    assert_has_kind("global x y", MatlabElementType::GlobalStmt);
    assert_has_kind("persistent z", MatlabElementType::PersistentStmt);
}

#[test]
fn test_parser_member_access() {
    assert_has_kind("containers.Map", MatlabElementType::MemberAccess);
    assert_has_kind("a.b.c", MatlabElementType::MemberAccess);
    assert_has_kind("containers.Map('a', 1)", MatlabElementType::Call);
}

#[test]
fn test_parser_end_index() {
    assert_has_kind("[1, 2, 3](end)", MatlabElementType::Call);
}

#[test]
fn test_parser_colon_all_index() {
    parse_ok("A(1,:)");
}
