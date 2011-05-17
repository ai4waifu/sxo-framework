//! Frontend number literal parse/render (SXO-owned; Athena holds [`Number`] only).

use athena::{
    numeric::{Number, Real, number_from_wire},
    types::WireNumber,
};

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

fn machine_f64(re: &Real) -> Option<f64> {
    match re {
        Real::Machine(x) => Some(*x),
        _ => None,
    }
}

fn near_zero(x: f64) -> bool {
    x == 0.0 || x.abs() < 1e-12
}

fn fmt_scalar(x: f64) -> String {
    if near_zero(x.fract()) && x.abs() < 1e15 { format!("{}", x as i64) } else { format!("{x}") }
}

fn render_machine_complex(re: f64, im: f64) -> String {
    if near_zero(re) {
        if near_zero(im) {
            return "0".into();
        }
        if near_zero(im - 1.0) {
            return "I".into();
        }
        if near_zero(im + 1.0) {
            return "-I".into();
        }
        return format!("{}*I", fmt_scalar(im));
    }
    if near_zero(im) {
        return fmt_scalar(re);
    }
    if near_zero(im - 1.0) {
        return format!("{}+I", fmt_scalar(re));
    }
    if near_zero(im + 1.0) {
        return format!("{}-I", fmt_scalar(re));
    }
    if im > 0.0 {
        format!("{}+{}*I", fmt_scalar(re), fmt_scalar(im))
    }
    else {
        format!("{}-{}*I", fmt_scalar(re), fmt_scalar(-im))
    }
}

/// Baseline numeric render (dialect renderers may override formatting).
pub fn render_number(n: &Number) -> String {
    if let Some(z) = n.as_complex() {
        if let (Some(re), Some(im)) = (machine_f64(&z.re), machine_f64(&z.im)) {
            return render_machine_complex(re, im);
        }
    }
    n.to_render_string()
}
