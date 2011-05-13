//! Oak `oak-wolfram` lexer golden and token tests (migrated from upstream oak.rs).

use oak_core::{LexerState, source::Source};
use oak_testing::lexing::LexerTester;
use oak_wolfram::{WolframLanguage, WolframLexer};
use std::{path::Path, time::Duration};

#[test]
#[ignore = "manual baseline regeneration helper; do not run in CI"]
fn generate_baseline() {
    use oak_core::{Lexer, ParseSession, SourceText, TokenType, source::Source};
    use serde_json::json;
    use std::fs;

    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = here.join("tests/fixtures/oak_wolfram_lexer");
    let source_path = fixture_dir.join("basic.wl");
    let source_text = fs::read_to_string(source_path)
        .expect("Failed to read source")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let source = SourceText::new(source_text);
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);
    let mut cache = ParseSession::default();
    let result = lexer.lex(&source, &[], &mut cache);

    let tokens = result.result.expect("Lexing failed");
    let token_data: Vec<_> = tokens
        .iter()
        .filter(|t| !t.kind.is_ignored())
        .map(|t| {
            let text = source.get_text_in(t.span.clone()).to_string();
            json!({
                "kind": format!("{:?}", t.kind),
                "text": text,
                "start": t.span.start,
                "end": t.span.end
            })
        })
        .collect();

    let output = json!({
        "success": true,
        "count": token_data.len(),
        "tokens": token_data,
        "errors": []
    });

    let output_path = fixture_dir.join("basic.wl.lexed.json");
    fs::write(output_path, serde_json::to_string_pretty(&output).unwrap()).expect("Failed to write baseline");
}

#[test]
fn test_slot_n_and_message_name_tokens() {
    use oak_core::{Lexer, ParseSession, SourceText, TokenType};

    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);
    let mut cache = ParseSession::default();

    let source = SourceText::new("#2 f::x ##3 ??Plus".to_string());
    let result = lexer.lex(&source, &[], &mut cache);
    let tokens = result.result.expect("lex");
    let kinds: Vec<_> = tokens
        .iter()
        .filter(|t| !t.kind.is_ignored())
        .map(|t| (format!("{:?}", t.kind), source.get_text_in(t.span.clone()).to_string()))
        .collect();

    assert!(
        kinds.iter().any(|(k, t)| k == "Slot" && t == "#2"),
        "expected Slot #2 token, got {kinds:?}"
    );
    assert!(
        kinds.iter().any(|(k, t)| k == "SlotSequence" && t == "##3"),
        "expected SlotSequence ##3 token, got {kinds:?}"
    );
    assert!(
        kinds.iter().any(|(k, _)| k == "MessageName"),
        "expected MessageName :: token, got {kinds:?}"
    );
    assert!(
        kinds.iter().any(|(k, _)| k == "DoubleQuestion"),
        "expected DoubleQuestion ?? token, got {kinds:?}"
    );
}

#[test]
fn test_wolfram_lexer() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);
    let test_runner = LexerTester::new(here.join("tests/fixtures/oak_wolfram_lexer"))
        .with_extension("wl")
        .with_timeout(Duration::from_secs(5));
    match test_runner.run_tests::<WolframLanguage, _>(&lexer) {
        Ok(()) => println!("Wolfram lexer tests passed!"),
        Err(e) => panic!("Wolfram lexer tests failed: {}", e),
    }
}

#[test]
fn test_peek_behavior() {
    use oak_core::SourceText;

    let source = SourceText::new("Module[{x}, x + 1]");
    let mut state = LexerState::<SourceText, WolframLanguage>::new(&source);

    state.advance(1);
    state.advance(1);
    assert_eq!(state.get_position(), 2);
}

#[test]
fn test_wolfram_function_parsing() {
    use oak_core::{Lexer, SourceText};

    let source = SourceText::new("Module[{x}, x + 1]");
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);

    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let result = lexer.lex(&source, &[], &mut cache);

    let tokens = result.result.expect("Lexing should succeed");
    assert!(!tokens.is_empty(), "Should parse at least one token");

    let first_token = &tokens[0];
    let token_text = source.get_text_in(first_token.span.clone());

    assert_eq!(token_text, "Module", "Identifier should be parsed as Module");
    assert_eq!(first_token.span.start, 0, "Token should start at position 0");
    assert_eq!(first_token.span.end, 6, "Token should end at position 6");
}
