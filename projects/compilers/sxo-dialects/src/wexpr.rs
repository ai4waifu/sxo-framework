//! Mathematica frontend form (`WExpr`) — Wolfram-shaped; **not** engine IR.

use std::fmt;

use athena::Number;

/// Atomic Wolfram-facing value.
#[derive(Debug, Clone, PartialEq)]
pub enum WAtom {
    /// Unified number (same tower as kernel).
    Number(Number),
    /// String literal.
    String(String),
    /// Symbol name.
    Symbol(String),
}

/// Wolfram-shaped tree for the Mathematica frontend (`Head[args…]`).
#[derive(Debug, Clone, PartialEq)]
pub enum WExpr {
    /// Atom.
    Atom(WAtom),
    /// `{a, b, …}`.
    List(Vec<WExpr>),
    /// `head[args…]`.
    Call {
        /// Head (usually a symbol).
        head: Box<WExpr>,
        /// Arguments.
        args: Vec<WExpr>,
    },
}

impl WExpr {
    /// Symbol atom.
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Atom(WAtom::Symbol(name.into()))
    }

    /// Small exact integer.
    pub fn int(n: i64) -> Self {
        Self::number(Number::small_int(n))
    }

    /// Number atom.
    pub fn number(n: Number) -> Self {
        Self::Atom(WAtom::Number(n))
    }

    /// Machine real (inexact).
    pub fn real(n: f64) -> Self {
        Self::number(Number::machine(n))
    }

    /// `head[args…]`.
    pub fn call(head: impl Into<String>, args: Vec<WExpr>) -> Self {
        Self::Call { head: Box::new(Self::symbol(head)), args }
    }

    /// Head symbol name, if any.
    pub fn head_name(&self) -> Option<&str> {
        match self {
            Self::Call { head, .. } => match head.as_ref() {
                Self::Atom(WAtom::Symbol(s)) => Some(s.as_str()),
                _ => None,
            },
            Self::List(_) => Some("List"),
            Self::Atom(WAtom::Symbol(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Lossy float — not for kernel semantics.
    pub fn as_f64_lossy(&self) -> Option<f64> {
        match self {
            Self::Atom(WAtom::Number(n)) => n.to_f64_lossy(),
            _ => None,
        }
    }

    /// Whether this is the given symbol.
    pub fn is_symbol(&self, name: &str) -> bool {
        matches!(self, Self::Atom(WAtom::Symbol(s)) if s == name)
    }

    /// Whether numeric `-1`.
    pub fn is_neg_one(&self) -> bool {
        matches!(self, Self::Atom(WAtom::Number(n)) if n.is_neg_one())
    }
}

impl fmt::Display for WExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::render_wexpr::render_wexpr(self))
    }
}
