//! Stateful host session for harness **load → bind → invoke** (no program stitching).

use std::rc::Rc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value as JsonValue;
use sxo_types::Dialect;

use crate::{
    dialects::{dialect_from_str, map_err},
    handles::{Expression, from_outcome},
    json::{bind_session_term, json_to_term, term_to_json},
    options::{EvaluateOptions, parse_strategy},
    session::Session,
};

fn invoke_source(dialect: Dialect, symbol: &str, arg_names: &[String]) -> String {
    match dialect {
        Dialect::Mathematica => {
            if arg_names.is_empty() {
                format!("{symbol}[]")
            } else {
                format!("{symbol}[{}]", arg_names.join(", "))
            }
        }
        Dialect::Matlab => {
            if arg_names.is_empty() {
                format!("{symbol}()")
            } else {
                format!("{symbol}({})", arg_names.join(", "))
            }
        }
        Dialect::SimpleMath => format!("{symbol}"),
    }
}

/// Persistent SXO session for conformance / bench (definitions + JSON argument bindings).
#[napi]
pub struct HostSession {
    inner: Rc<Session>,
    dialect: Dialect,
}

#[napi]
impl HostSession {
    /// `dialect`: `mathematica` | `matlab` | `simple-math`.
    #[napi(constructor)]
    pub fn new(dialect: Option<String>) -> Result<Self> {
        Ok(Self { inner: Rc::new(Session::new()), dialect: dialect_from_str(dialect)? })
    }

    /// Parse and evaluate a definition script (`:=` / `function`); discards the returned value.
    #[napi(js_name = "evaluateDefinition")]
    pub fn evaluate_definition(&self, source: String, options: Option<EvaluateOptions>) -> Result<()> {
        let strategy = parse_strategy(&options)?;
        let outcome = self.inner.evaluate_input(&source, self.dialect).map_err(map_err)?;
        let _ = from_outcome(Rc::clone(&self.inner), self.dialect, outcome, strategy)?;
        Ok(())
    }

    /// Bind a harness JSON value to a session symbol (no surface literal).
    #[napi(js_name = "bindJson")]
    pub fn bind_json(&self, name: String, json: String) -> Result<()> {
        let value: JsonValue = serde_json::from_str(&json).map_err(|e| Error::from_reason(format!("invalid harness json: {e}")))?;
        self.inner
            .with_math_mut(|ms| {
                let term = json_to_term(ms, &value)?;
                bind_session_term(ms, &name, term)
            })
            .map_err(map_err)
    }

    /// Call `symbol` with previously bound argument names.
    #[napi]
    pub fn invoke(&self, symbol: String, arg_names: Vec<String>, options: Option<EvaluateOptions>) -> Result<Expression> {
        let strategy = parse_strategy(&options)?;
        let source = invoke_source(self.dialect, &symbol, &arg_names);
        let outcome = self.inner.evaluate_input(&source, self.dialect).map_err(map_err)?;
        from_outcome(Rc::clone(&self.inner), self.dialect, outcome, strategy)
    }

    /// Project an evaluate result to harness JSON string.
    #[napi(js_name = "termToJson")]
    pub fn term_to_json(&self, expr: &Expression) -> Result<String> {
        let term = expr.project_symbolic_term()?;
        let value = self.inner.with_math(|ms| term_to_json(ms, term)).map_err(map_err)?;
        serde_json::to_string(&value).map_err(|e| Error::from_reason(format!("term_to_json encode: {e}")))
    }

    /// Drop all Own definitions on this session.
    #[napi(js_name = "clearDefinitions")]
    pub fn clear_definitions(&self) {
        self.inner.clear_definitions();
    }
}
