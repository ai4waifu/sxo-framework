//! MATLAB dialect Form (`MatlabForm`) — dialect-shaped, not Athena IR and not `WExpr`.
//!
//! Living `14`: `typed AST → MatlabForm → Athena request`. Term arena materialization is a
//! transitional bridge in [`crate::lower`], not the long-term Form identity.

use athena::{
    numeric::{Number, to_f64_lossy},
    runtime::values::numeric_clone::clone_number,
};

/// Atomic MATLAB-facing value.
#[derive(Debug, PartialEq)]
pub enum MatlabAtom {
    /// Unified number (same tower as kernel).
    Number(Number),
    /// String literal.
    String(String),
    /// Symbol / identifier name.
    Symbol(String),
    /// Boolean literal.
    Bool(bool),
    /// Empty / missing branch placeholder (`[]`-adjacent null in control trees).
    Null,
}

impl Clone for MatlabAtom {
    fn clone(&self) -> Self {
        match self {
            Self::Number(n) => Self::Number(clone_number(n)),
            Self::String(s) => Self::String(s.clone()),
            Self::Symbol(s) => Self::Symbol(s.clone()),
            Self::Bool(b) => Self::Bool(*b),
            Self::Null => Self::Null,
        }
    }
}

/// MATLAB dialect Form tree (surface heads + lists).
#[derive(Debug, Clone, PartialEq)]
pub enum MatlabForm {
    /// Atom.
    Atom(MatlabAtom),
    /// Row / nested array payload as a list of forms.
    List(Vec<MatlabForm>),
    /// Dialect surface call `head(args…)` (may later map to Semantic or Extension).
    Call {
        /// Surface head name (`Plus`, `Set`, `If`, `sin` mapped heads, …).
        head: String,
        /// Arguments.
        args: Vec<MatlabForm>,
    },
}

impl MatlabForm {
    /// Symbol atom.
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Atom(MatlabAtom::Symbol(name.into()))
    }

    /// Boolean atom.
    pub fn bool(value: bool) -> Self {
        Self::Atom(MatlabAtom::Bool(value))
    }

    /// Null atom.
    pub fn null() -> Self {
        Self::Atom(MatlabAtom::Null)
    }

    /// Small exact integer.
    pub fn int(n: i64) -> Self {
        Self::number(Number::small_int(n))
    }

    /// Number atom.
    pub fn number(n: Number) -> Self {
        Self::Atom(MatlabAtom::Number(n))
    }

    /// String atom.
    pub fn string(s: impl Into<String>) -> Self {
        Self::Atom(MatlabAtom::String(s.into()))
    }

    /// List of forms.
    pub fn list(items: Vec<MatlabForm>) -> Self {
        Self::List(items)
    }

    /// Surface call.
    pub fn call(head: impl Into<String>, args: Vec<MatlabForm>) -> Self {
        Self::Call { head: head.into(), args }
    }

    /// Head name for a call, if any.
    pub fn head_name(&self) -> Option<&str> {
        match self {
            Self::Call { head, .. } => Some(head.as_str()),
            Self::List(_) => Some("List"),
            Self::Atom(MatlabAtom::Symbol(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Whether this is the given symbol.
    pub fn is_symbol(&self, name: &str) -> bool {
        matches!(self, Self::Atom(MatlabAtom::Symbol(s)) if s == name)
    }

    /// Lossy float — not for kernel semantics.
    pub fn as_f64_lossy(&self) -> Option<f64> {
        match self {
            Self::Atom(MatlabAtom::Number(n)) => to_f64_lossy(n),
            _ => None,
        }
    }
}
