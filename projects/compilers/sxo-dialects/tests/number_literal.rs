//! Integration tests for number_literal.

use sxo_dialects::{Number, parse_number_literal};

#[test]
fn parse_integer_and_decimal() {
    assert_eq!(parse_number_literal("42").unwrap(), Number::small_int(42));
    assert!(parse_number_literal("1.5").unwrap().to_f64_lossy().unwrap() > 1.4);
}
