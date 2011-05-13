//! Oak `oak-wolfram` builder smoke tests (migrated from upstream oak.rs).

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_wolfram::{WolframBuilder, WolframLanguage};

#[test]
fn test_functional_parsing() {
    let language = WolframLanguage::default();
    let builder = WolframBuilder::new(&language);
    let mut session = ParseSession::<WolframLanguage>::default();

    let code = "f @ x + g /@ list // h";
    let source = SourceText::new(code);
    let output = builder.build(&source, &[], &mut session);

    assert!(output.result.is_ok());
}

#[test]
fn test_pure_function() {
    let language = WolframLanguage::default();
    let builder = WolframBuilder::new(&language);
    let mut session = ParseSession::<WolframLanguage>::default();

    let code = "# + 1 &";
    let source = SourceText::new(code);
    let output = builder.build(&source, &[], &mut session);

    assert!(output.result.is_ok());
}

#[test]
fn test_apply_level_and_map_all() {
    let language = WolframLanguage::default();
    let builder = WolframBuilder::new(&language);
    let mut session = ParseSession::<WolframLanguage>::default();

    let code = "f @@@ terms + g //@ list";
    let source = SourceText::new(code);
    let output = builder.build(&source, &[], &mut session);

    assert!(output.result.is_ok());
}

#[test]
fn test_nested_calls() {
    let language = WolframLanguage::default();
    let builder = WolframBuilder::new(&language);
    let mut session = ParseSession::<WolframLanguage>::default();

    let code = "f[x][y][z]";
    let source = SourceText::new(code);
    let output = builder.build(&source, &[], &mut session);

    assert!(output.result.is_ok());
}
