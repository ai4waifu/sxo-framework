//! Mathematica dialect Form (`WolframForm`) — Wolfram-shaped, not engine IR.

use std::fmt;

use athena::{
    numeric::{Number, to_f64_lossy},
    runtime::values::numeric_clone::clone_number,
};

/// Atomic Wolfram-facing value.
#[derive(Debug, PartialEq)]
pub enum WolframAtom {
    /// Unified number (same tower as kernel).
    Number(Number),
    /// String literal.
    String(String),
    /// Symbol name.
    Symbol(String),
}

impl Clone for WolframAtom {
    fn clone(&self) -> Self {
        match self {
            Self::Number(n) => Self::Number(clone_number(n)),
            Self::String(s) => Self::String(s.clone()),
            Self::Symbol(s) => Self::Symbol(s.clone()),
        }
    }
}

/// Wolfram-shaped tree for the Mathematica frontend (`Head[args…]`).
#[derive(Debug, Clone, PartialEq)]
pub enum WolframForm {
    /// Atom.
    Atom(WolframAtom),
    /// `{a, b, …}`.
    List(Vec<WolframForm>),
    /// `head[args…]`.
    Call {
        /// Head (usually a symbol).
        head: Box<WolframForm>,
        /// Arguments.
        args: Vec<WolframForm>,
    },
}

impl WolframForm {
    /// Symbol atom.
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Atom(WolframAtom::Symbol(name.into()))
    }

    /// Small exact integer.
    pub fn int(n: i64) -> Self {
        Self::number(Number::small_int(n))
    }

    /// Number atom.
    pub fn number(n: Number) -> Self {
        Self::Atom(WolframAtom::Number(n))
    }

    /// Machine real (inexact).
    pub fn real(n: f64) -> Self {
        Self::number(Number::machine(n))
    }

    /// `head[args…]`.
    pub fn call(head: impl Into<String>, args: Vec<WolframForm>) -> Self {
        Self::Call { head: Box::new(Self::symbol(head)), args }
    }

    /// Head symbol name, if any.
    pub fn head_name(&self) -> Option<&str> {
        match self {
            Self::Call { head, .. } => match head.as_ref() {
                Self::Atom(WolframAtom::Symbol(s)) => Some(s.as_str()),
                _ => None,
            },
            Self::List(_) => Some("List"),
            Self::Atom(WolframAtom::Symbol(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Lossy float — not for kernel semantics.
    pub fn as_f64_lossy(&self) -> Option<f64> {
        match self {
            Self::Atom(WolframAtom::Number(n)) => to_f64_lossy(n),
            _ => None,
        }
    }

    /// Whether this is the given symbol.
    pub fn is_symbol(&self, name: &str) -> bool {
        matches!(self, Self::Atom(WolframAtom::Symbol(s)) if s == name)
    }

    /// Whether numeric `-1`.
    pub fn is_neg_one(&self) -> bool {
        matches!(self, Self::Atom(WolframAtom::Number(n)) if n.is_neg_one())
    }
}

impl fmt::Display for WolframForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::render::render(self))
    }
}
