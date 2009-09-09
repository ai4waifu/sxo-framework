//! Bridge IR [`Term`] — frontend lowered tree (oak / WExpr / MATLAB), not kernel IR.

use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;

use euler::Number;

use crate::number_literal::parse_number_literal;

/// Extract kernel number from a bridge term atom.
pub fn number_from_term(term: &Term) -> Option<&Number> {
    match term {
        Term::Atom(Atom::Number(n)) => Some(n),
        _ => None,
    }
}

/// Atomic value in the engine IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// Unified kernel number (sole numeric truth source).
    Number(Number),
    /// String literal.
    String(String),
    /// Symbol name.
    Symbol(String),
}

/// Bridge expression tree (parse / render / transitional eval).
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// Atom.
    Atom(Atom),
    /// Ordered collection (lists / arrays at the engine layer).
    List(Vec<Term>),
    /// Application `head(args…)`.
    App {
        /// Head term (usually a symbol).
        head: Box<Term>,
        /// Arguments.
        args: Vec<Term>,
    },
}

impl Term {
    /// Symbol atom.
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Atom(Atom::Symbol(name.into()))
    }

    /// Small exact integer convenience.
    pub fn int(n: i64) -> Self {
        Self::number(Number::small_int(n))
    }

    /// Arbitrary-precision exact integer.
    pub fn integer(n: impl Into<BigInt>) -> Self {
        Self::number(Number::integer(n))
    }

    /// Exact rational (normalized).
    pub fn rational(r: BigRational) -> Self {
        Self::number(Number::rational(r))
    }

    /// Machine real from inexact float literal.
    pub fn real(n: f64) -> Self {
        Self::number(Number::machine(n))
    }

    /// Unified number atom.
    pub fn number(n: Number) -> Self {
        Self::Atom(Atom::Number(n))
    }

    /// Parse numeric literal (exact int unless decimal/exponent → machine).
    pub fn from_number_literal(text: &str) -> Option<Self> {
        parse_number_literal(text).map(Self::number)
    }

    /// `head(args…)` with symbol head.
    pub fn app(head: impl Into<String>, args: Vec<Term>) -> Self {
        Self::App { head: Box::new(Self::symbol(head)), args }
    }

    /// Head symbol name, if any.
    pub fn head_name(&self) -> Option<&str> {
        match self {
            Self::App { head, .. } => match head.as_ref() {
                Self::Atom(Atom::Symbol(s)) => Some(s.as_str()),
                _ => None,
            },
            Self::List(_) => Some("List"),
            Self::Atom(Atom::Symbol(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Kernel number reference (no precision loss).
    pub fn as_number(&self) -> Option<&Number> {
        number_from_term(self)
    }

    /// Lossy `f64` — legacy bridge / display hints only.
    pub fn as_f64_lossy(&self) -> Option<f64> {
        self.as_number().and_then(Number::to_f64_lossy)
    }

    /// Whether this is the given symbol.
    pub fn is_symbol(&self, name: &str) -> bool {
        matches!(self, Self::Atom(Atom::Symbol(s)) if s == name)
    }

    /// Whether numeric `-1`.
    pub fn is_neg_one(&self) -> bool {
        self.as_number().is_some_and(Number::is_neg_one)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
