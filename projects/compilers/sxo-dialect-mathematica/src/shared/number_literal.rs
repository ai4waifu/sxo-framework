//! Frontend number literal parse/render (SXO-owned; Athena holds [`Number`] only).

use athena::{
    numeric::Number,
    numeric::number_from_wire,
};
use athena_types::WireNumber;

/// Parse source text into kernel [`Number`] via wire (integer, rational, or machine float).
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
    else if let Some((numer, denom)) = source.split_once('/') {
        let numer: i64 = numer.trim().parse().ok()?;
        let denom: i64 = denom.trim().parse().ok()?;
        let wire = WireNumber::rational_i64(numer, denom).ok()?;
        number_from_wire(&wire).ok()
    }
    else {
        let wire = WireNumber::from_decimal_str(source)?;
        number_from_wire(&wire).ok()
    }
}

/// Baseline numeric render (dialect renderers may override formatting).
pub fn render_number(n: &Number) -> String {
    n.to_render_string()
}
