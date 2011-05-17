//! Oak `oak-wolfram` parser conformance tests (migrated from upstream oak.rs).

use oak_core::{Lexer, Parser, source::SourceText};
use oak_wolfram::{WolframLanguage, WolframLexer, WolframParser};

#[test]
fn test_parser_basic() {
    let input = "f[x, {1, 2, 3}]";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);

    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let _lex_output = lexer.lex(&source, &[], &mut cache);

    let parser = WolframParser::new(&language);
    let diagnostics = parser.parse(&source, &[], &mut cache);

    assert!(diagnostics.result.is_ok())
}

#[test]
fn test_parser_binary_expr() {
    let input = "a + b * c";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    assert!(output.result.is_ok());
}

#[test]
fn test_parser_composition_ops() {
    for input in ["f@*g", "f/*g", "a@*b@*c", "a/*b/*c"] {
        let source = SourceText::new(input.to_string());
        let language = WolframLanguage::default();
        let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
        let parser = WolframParser::new(&language);
        let output = parser.parse(&source, &[], &mut cache);
        assert!(output.result.is_ok(), "parse failed for {input}");
    }
}

#[test]
fn test_parser_message_name_and_slot_n() {
    for input in ["f::x", "Message[f::x]", "#2", "#2 &", "MapIndexed[#2 &, {a, b}]", "??Plus"] {
        let source = SourceText::new(input.to_string());
        let language = WolframLanguage::default();
        let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
        let parser = WolframParser::new(&language);
        let output = parser.parse(&source, &[], &mut cache);
        assert!(output.result.is_ok(), "parse failed for {input}");
    }
}

#[test]
fn test_parser_derivative_prime() {
    for input in ["y'", "y'[x]", "DSolveValue[y'[x] == y[x], y[x], x]"] {
        let source = SourceText::new(input.to_string());
        let language = WolframLanguage::default();
        let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
        let parser = WolframParser::new(&language);
        let output = parser.parse(&source, &[], &mut cache);
        assert!(output.result.is_ok(), "parse failed for {input}");
    }
}

#[test]
fn test_parser_complex() {
    let input = "f[x] + g[y, {1, 2}] * (a + b)!";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    assert!(output.result.is_ok());
}

#[test]
fn test_parser_compound_expression() {
    let input = "a; b; c";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    assert!(output.result.is_ok());
}

#[test]
fn test_parser_root_semicolon() {
    let input = "1 + 2; 3 * 4";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    assert!(output.result.is_ok());
}

#[test]
fn test_parser_keyword_call_if() {
    let input = "If[1 == 1, 7, 8]";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    let root = output.result.expect("parse If call");
    let calls = count_kind(&root, oak_wolfram::parser::element_type::WolframElementType::Call);
    assert!(calls >= 1, "expected Call node for If[…], got root={root:?}");
}

#[test]
fn test_parser_keyword_call_import() {
    let input = "Import[\"x.csv\"]";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    let root = output.result.expect("parse Import call");
    let calls = count_kind(&root, oak_wolfram::parser::element_type::WolframElementType::Call);
    assert!(calls >= 1, "expected Call node for Import[…], got root={root:?}");
}

#[test]
fn test_parser_part_double_bracket() {
    let input = "{1, 2, 3}[[0]]";
    let source = SourceText::new(input.to_string());
    let language = WolframLanguage::default();
    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let parser = WolframParser::new(&language);
    let output = parser.parse(&source, &[], &mut cache);
    let root = output.result.expect("parse Part");
    let parts = count_kind(&root, oak_wolfram::parser::element_type::WolframElementType::Part);
    assert!(parts >= 1, "expected Part node for list[[0]], got root={root:?}");
}

fn count_kind(
    node: &oak_core::tree::GreenNode<'_, oak_wolfram::WolframLanguage>,
    kind: oak_wolfram::parser::element_type::WolframElementType,
) -> usize {
    use oak_core::tree::GreenTree;
    let mut n = if node.kind == kind { 1 } else { 0 };
    for child in node.children {
        if let GreenTree::Node(c) = child {
            n += count_kind(c, kind);
        }
    }
    n
}
