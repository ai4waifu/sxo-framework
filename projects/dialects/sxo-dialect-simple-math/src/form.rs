//! Simple Math dialect Form (`Expr`) — flat owned tree, not Athena IR and not `WExpr`.
//!
//! Surface intent (Living `05`): lowercase calls, `[…]` list, `{k:v}` dict.
//! oak language: `oak-athena`. SXO product tags: `simple-math` / `sm` (optional `sxo`).
//! Off current delivery route. When re-enabled: `oak-athena` → `Expr` → lower only.

use std::fmt;

/// Dict key in `{ k: v }` literals (identifier or string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictKey {
    /// Bare identifier key: `{ a: 1 }`.
    Ident(String),
    /// String key: `{ "a": 1 }`.
    String(String),
}

/// Symbolic expression tree (Simple Math surface shape).
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Num(f64),
    /// Variable / identifier name (lowercase preferred for builtins).
    Var(String),
    /// Unary negation.
    Neg(Box<Expr>),
    /// Addition.
    Add(Box<Expr>, Box<Expr>),
    /// Subtraction.
    Sub(Box<Expr>, Box<Expr>),
    /// Multiplication.
    Mul(Box<Expr>, Box<Expr>),
    /// Division.
    Div(Box<Expr>, Box<Expr>),
    /// Exponentiation.
    Pow(Box<Expr>, Box<Expr>),
    /// Lowercase call: `sin(x)`, `diff(f, x)`, …
    Call {
        /// Surface head (must be lowercase for builtins).
        head: String,
        /// Arguments.
        args: Vec<Expr>,
    },
    /// List literal: `[a, b, c]`.
    List(Vec<Expr>),
    /// Dict literal: `{ a: 1, "b": 2 }`.
    Dict(Vec<(DictKey, Expr)>),
}

impl Expr {
    /// Numeric constant.
    pub fn num(v: f64) -> Self {
        Self::Num(v)
    }

    /// Variable.
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var(name.into())
    }

    /// `a + b`
    #[allow(clippy::should_implement_trait)]
    pub fn add(a: Expr, b: Expr) -> Self {
        Self::Add(Box::new(a), Box::new(b))
    }

    /// `a - b`
    #[allow(clippy::should_implement_trait)]
    pub fn sub(a: Expr, b: Expr) -> Self {
        Self::Sub(Box::new(a), Box::new(b))
    }

    /// `a * b`
    #[allow(clippy::should_implement_trait)]
    pub fn mul(a: Expr, b: Expr) -> Self {
        Self::Mul(Box::new(a), Box::new(b))
    }

    /// `a / b`
    #[allow(clippy::should_implement_trait)]
    pub fn div(a: Expr, b: Expr) -> Self {
        Self::Div(Box::new(a), Box::new(b))
    }

    /// `a ^ b`
    pub fn pow(a: Expr, b: Expr) -> Self {
        Self::Pow(Box::new(a), Box::new(b))
    }

    /// `-a`
    #[allow(clippy::should_implement_trait)]
    pub fn neg(a: Expr) -> Self {
        Self::Neg(Box::new(a))
    }

    /// Lowercase call `head(args…)`.
    pub fn call(head: impl Into<String>, args: Vec<Expr>) -> Self {
        Self::Call { head: head.into(), args }
    }

    /// `sin(a)` sugar → lowercase [`Self::Call`].
    pub fn sin(a: Expr) -> Self {
        Self::call("sin", vec![a])
    }

    /// `cos(a)` sugar → lowercase [`Self::Call`].
    pub fn cos(a: Expr) -> Self {
        Self::call("cos", vec![a])
    }

    /// List literal.
    pub fn list(items: Vec<Expr>) -> Self {
        Self::List(items)
    }

    /// Dict literal.
    pub fn dict(entries: Vec<(DictKey, Expr)>) -> Self {
        Self::Dict(entries)
    }

    /// Whether this node is exactly numeric zero.
    pub fn is_zero(&self) -> bool {
        matches!(self, Self::Num(n) if *n == 0.0)
    }

    /// Whether this node is exactly numeric one.
    pub fn is_one(&self) -> bool {
        matches!(self, Self::Num(n) if *n == 1.0)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::render::render(self))
    }
}
