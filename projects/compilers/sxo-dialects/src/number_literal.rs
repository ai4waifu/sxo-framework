//! Frontend number literal parse/render (SXO-owned; Athena holds [`Number`] only).

use athena::{ExactNumber, Number, RealNumber, Term};
use num_bigint::BigInt;

/// Parse source text into kernel [`Number`] (integer → exact; decimal/exponent → machine).
pub fn parse_number_literal(text: &str) -> Option<Number> {
    let source = text.trim();
    if source.is_empty() {
        return None;
    }
    let is_machine =
        source.contains('.') || source.contains('e') || source.contains('E') || (source.ends_with('.') && source.len() > 1);
    if is_machine {
        let n: f64 = source.parse().ok()?;
        Some(Number::machine(n))
    }
    else if let Ok(n) = source.parse::<BigInt>() {
        Some(Number::integer(n))
    }
    else {
        None
    }
}

/// Parse a numeric literal into an Athena [`Term`] number atom.
pub fn term_from_number_literal(text: &str) -> Option<Term> {
    parse_number_literal(text).map(Term::number)
}

/// Baseline numeric render (dialect renderers may override formatting).
pub fn render_number(n: &Number) -> String {
    match n {
        Number::Exact(ExactNumber::Integer(i)) => i.to_string(),
        Number::Exact(ExactNumber::Rational(r)) => format!("{}/{}", r.numer(), r.denom()),
        Number::Real(RealNumber::Machine(x)) => format_machine(*x),
    }
}

fn format_machine(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) } else { format!("{n}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer_and_decimal() {
        assert_eq!(parse_number_literal("42").unwrap(), Number::small_int(42));
        assert!(parse_number_literal("1.5").unwrap().to_f64_lossy().unwrap() > 1.4);
    }
}
