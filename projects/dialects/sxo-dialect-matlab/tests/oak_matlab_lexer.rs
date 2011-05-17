//! Oak `oak-matlab` lexer golden tests (migrated from upstream oak.rs).

use oak_matlab::{MatlabLanguage, MatlabLexer};
use oak_testing::lexing::LexerTester;
use std::{path::Path, time::Duration};

#[test]
fn test_matlab_lexer() -> Result<(), oak_core::OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = MatlabLanguage::default();
    let lexer = MatlabLexer::new(&language);
    let test_runner =
        LexerTester::new(here.join("tests/fixtures/oak_matlab_lexer")).with_extension("m").with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&lexer)
}
