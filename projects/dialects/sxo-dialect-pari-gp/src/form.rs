//! PARI/GP dialect Form (`GpForm`) — dialect-shaped, not Athena IR.
//!
//! Target path: `oak-pari` typed AST → [`GpForm`] → Athena request.
//! This scaffold only defines the Form shape so the product package and matrix
//! have a stable Rust counterpart.

use std::fmt;

/// Atomic GP-facing value (placeholder; exact number tower lands with Athena).
#[derive(Debug, Clone, PartialEq)]
pub enum GpAtom {
    /// Decimal / integer literal text as written in source (exact parse deferred).
    NumberLiteral(String),
    /// String literal.
    String(String),
    /// Identifier / GEN name.
    Symbol(String),
    /// Boolean (`1` / `0` in GP often; keep explicit for Form).
    Bool(bool),
}

/// PARI/GP dialect Form tree (surface heads + lists).
#[derive(Debug, Clone, PartialEq)]
pub enum GpForm {
    /// Atom.
    Atom(GpAtom),
    /// Vector / list payload.
    List(Vec<GpForm>),
    /// Surface call `head(args…)` (GP function or operator head).
    Call {
        /// Surface head (`Plus`, `factor`, `Mod`, …).
        head: String,
        /// Arguments.
        args: Vec<GpForm>,
    },
}

impl GpForm {
    /// Symbol atom.
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Atom(GpAtom::Symbol(name.into()))
    }

    /// Boolean atom.
    pub fn bool(value: bool) -> Self {
        Self::Atom(GpAtom::Bool(value))
    }

    /// Number literal atom (source text preserved).
    pub fn number_literal(text: impl Into<String>) -> Self {
        Self::Atom(GpAtom::NumberLiteral(text.into()))
    }

    /// String atom.
    pub fn string(s: impl Into<String>) -> Self {
        Self::Atom(GpAtom::String(s.into()))
    }

    /// List / vector.
    pub fn list(items: Vec<GpForm>) -> Self {
        Self::List(items)
    }

    /// Surface call.
    pub fn call(head: impl Into<String>, args: Vec<GpForm>) -> Self {
        Self::Call { head: head.into(), args }
    }

    /// Symbol name when this form is a symbol atom.
    pub fn head_name(&self) -> Option<&str> {
        match self {
            Self::Atom(GpAtom::Symbol(s)) => Some(s.as_str()),
            Self::Call { head, .. } => Some(head.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for GpForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::render::render_gp(self))
    }
}
