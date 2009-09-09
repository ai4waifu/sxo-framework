//! Owned expression AST for SXO (no third-party E-Graph).

use std::fmt;

/// Symbolic expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Num(f64),
    /// Variable name.
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
    /// Sine.
    Sin(Box<Expr>),
    /// Cosine.
    Cos(Box<Expr>),
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

    /// `sin(a)`
    pub fn sin(a: Expr) -> Self {
        Self::Sin(Box::new(a))
    }

    /// `cos(a)`
    pub fn cos(a: Expr) -> Self {
        Self::Cos(Box::new(a))
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
        write!(f, "{}", crate::render::render_simple_math(self))
    }
}
