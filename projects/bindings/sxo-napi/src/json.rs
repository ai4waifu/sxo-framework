//! Harness / product JSON ↔ arena [`TermId`] marshalling (no dialect surface syntax).

use athena::{
    api::{AthenaRequest, SessionCommand},
    ir::{Atom, TermBuilder, TermNode},
    numeric::{NumericValue, to_f64_lossy},
    runtime::values::arena::{default_span, push_bool, push_int, push_list, push_null},
    types::{BindingEvaluationPolicy, BindingKind, TermId},
    Session as AthenaSession,
};
use serde_json::Value as JsonValue;
use sxo_types::SxoError;

/// LeetCode `metadata.tests` JSON → session arena term (recursive list / scalar).
pub fn json_to_term(session: &mut AthenaSession, value: &JsonValue) -> Result<TermId, SxoError> {
    match value {
        JsonValue::Null => Ok(push_null(session)),
        JsonValue::Bool(b) => Ok(push_bool(session, *b)),
        JsonValue::Number(n) => json_number_to_term(session, n),
        JsonValue::String(s) => {
            let mut b = TermBuilder::new(&mut session.arena);
            Ok(b.string(s.clone(), default_span()))
        }
        JsonValue::Array(items) => {
            let mut terms = Vec::with_capacity(items.len());
            for item in items {
                terms.push(json_to_term(session, item)?);
            }
            Ok(push_list(session, terms))
        }
        JsonValue::Object(_) => Err(SxoError::new("json_object_not_supported_in_harness_contract")),
    }
}

fn json_number_to_term(session: &mut AthenaSession, n: &serde_json::Number) -> Result<TermId, SxoError> {
    if let Some(i) = n.as_i64() {
        return Ok(push_int(session, i));
    }
    if let Some(u) = n.as_u64() {
        if u <= i64::MAX as u64 {
            return Ok(push_int(session, u as i64));
        }
    }
    if let Some(f) = n.as_f64() {
        if f.is_finite() && f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
            return Ok(push_int(session, f as i64));
        }
        let mut b = TermBuilder::new(&mut session.arena);
        let numeric = NumericValue::decimal_from_f64(f).map_err(|e| SxoError::new(e.to_string()))?;
        return Ok(b.number(numeric, default_span()));
    }
    Err(SxoError::new("json_number_not_finite"))
}

/// Project a symbolic term back to harness JSON (scalar / null / flat number list).
pub fn term_to_json(session: &AthenaSession, term: TermId) -> Result<JsonValue, SxoError> {
    match session.arena.get(term) {
        Some(TermNode::Atom(Atom::Null)) => Ok(JsonValue::Null),
        Some(TermNode::Atom(Atom::Boolean(v))) => Ok(JsonValue::Bool(*v)),
        Some(TermNode::Atom(Atom::Number(n))) => number_to_json(n),
        Some(TermNode::Atom(Atom::String(s))) => Ok(JsonValue::String(s.clone())),
        Some(TermNode::Collection { elements, .. }) => {
            let mut out = Vec::with_capacity(elements.len());
            for id in elements {
                out.push(term_to_json(session, *id)?);
            }
            Ok(JsonValue::Array(out))
        }
        Some(_) => Err(SxoError::new("term_not_json_surface")),
        None => Err(SxoError::new("term_out_of_range")),
    }
}

fn number_to_json(n: &NumericValue) -> Result<JsonValue, SxoError> {
    if let Some(i) = n.as_exact_integer() {
        return Ok(JsonValue::from(i));
    }
    if let Some(f) = to_f64_lossy(n) {
        return Ok(JsonValue::from(f));
    }
    Err(SxoError::new("number_not_json_representable"))
}

/// Bind an interned harness name to an already-built term (session Own).
pub fn bind_session_term(session: &mut AthenaSession, name: &str, value: TermId) -> Result<(), SxoError> {
    let symbol = session.arena.symbols_mut().intern(name);
    let request = AthenaRequest::Command(SessionCommand::Define {
        symbol,
        value,
        kind: BindingKind::Session,
        // Harness JSON is already materialized in the arena — capture without VM ConstructCollection.
        evaluation: BindingEvaluationPolicy::StoreResidualTerm,
    });
    athena::AthenaEngine::new()
        .execute_request(session, request)
        .map_err(SxoError::from_diagnostic)?;
    Ok(())
}
