//! Shared N-API evaluate options (strategy, not dialect-named modes).

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Options for host `evaluate` (one crossing; strategy applied in Rust).
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct EvaluateOptions {
    /// `"none"` (default) or `"simplify"`.
    pub strategy: Option<String>,
}

/// Neutral evaluate post-strategy (Athena Simplify head when requested).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvalStrategy {
    /// Return the evaluate result as-is.
    None,
    /// Project symbolic term and run engine `Simplify` before returning.
    Simplify,
}

/// Parse [`EvaluateOptions::strategy`].
pub(crate) fn parse_strategy(options: &Option<EvaluateOptions>) -> Result<EvalStrategy> {
    match options.as_ref().and_then(|o| o.strategy.as_deref()) {
        None | Some("none") => Ok(EvalStrategy::None),
        Some("simplify") => Ok(EvalStrategy::Simplify),
        Some(other) => Err(Error::from_reason(format!(
            "unknown evaluate strategy `{other}` (expected `none` or `simplify`)"
        ))),
    }
}
