//! Shared WASM evaluate strategy (neutral — not dialect-named modes).

use wasm_bindgen::JsValue;

/// Neutral evaluate post-strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvalStrategy {
    None,
    Simplify,
}

/// Parse `"none"` | `"simplify"` (default none).
pub(crate) fn parse_strategy(strategy: Option<&str>) -> Result<EvalStrategy, JsValue> {
    match strategy {
        None | Some("none") => Ok(EvalStrategy::None),
        Some("simplify") => Ok(EvalStrategy::Simplify),
        Some(other) => Err(JsValue::from_str(&format!("unknown evaluate strategy `{other}` (expected `none` or `simplify`)"))),
    }
}
