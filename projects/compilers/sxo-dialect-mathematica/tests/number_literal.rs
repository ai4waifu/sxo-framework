//! Integration tests for number literals.

use athena::{
    numeric::Number,
    numeric::to_f64_lossy,
};
use sxo_dialect_mathematica::parse_number_literal;

#[test]
fn parse_integer_and_decimal() {
    assert_eq!(parse_number_literal("42").unwrap(), Number::small_int(42));
    assert!(to_f64_lossy(&parse_number_literal("1.5").unwrap()).unwrap() > 1.4);
}

#[test]
fn parse_exact_rational() {
    let n = parse_number_literal("3/4").unwrap();
    assert_eq!(n.to_render_string(), "3/4");
}
