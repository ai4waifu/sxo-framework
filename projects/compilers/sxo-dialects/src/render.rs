//! Dual-dialect rendering.

use crate::expr::Expr;

/// Render as simple-math text.
pub fn render_simple_math(expr: &Expr) -> String {
    render(expr, Style::SimpleMath)
}

/// Render as Mathematica-like form.
pub fn render_mathematica(expr: &Expr) -> String {
    render(expr, Style::Mathematica)
}

#[derive(Clone, Copy)]
enum Style {
    SimpleMath,
    Mathematica,
}

fn render(expr: &Expr, style: Style) -> String {
    match expr {
        Expr::Num(n) => format_num(*n),
        Expr::Var(v) => v.clone(),
        Expr::Neg(a) => format!("-{}", maybe_paren(a, Prec::Unary, style)),
        Expr::Add(a, b) => format!("{} + {}", maybe_paren(a, Prec::Add, style), maybe_paren(b, Prec::Add, style)),
        Expr::Sub(a, b) => format!("{} - {}", maybe_paren(a, Prec::Add, style), maybe_paren(b, Prec::Mul, style)),
        Expr::Mul(a, b) => format!("{}*{}", maybe_paren(a, Prec::Mul, style), maybe_paren(b, Prec::Mul, style)),
        Expr::Div(a, b) => format!("{}/{}", maybe_paren(a, Prec::Mul, style), maybe_paren(b, Prec::Pow, style)),
        Expr::Pow(a, b) => format!("{}^{}", maybe_paren(a, Prec::Pow, style), maybe_paren(b, Prec::Pow, style)),
        Expr::Sin(a) => match style {
            Style::SimpleMath => format!("sin({})", render(a, style)),
            Style::Mathematica => format!("Sin[{}]", render(a, style)),
        },
        Expr::Cos(a) => match style {
            Style::SimpleMath => format!("cos({})", render(a, style)),
            Style::Mathematica => format!("Cos[{}]", render(a, style)),
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Add = 1,
    Mul = 2,
    Pow = 3,
    Unary = 4,
    Atom = 5,
}

fn prec(expr: &Expr) -> Prec {
    match expr {
        Expr::Num(_) | Expr::Var(_) | Expr::Sin(_) | Expr::Cos(_) => Prec::Atom,
        Expr::Neg(_) => Prec::Unary,
        Expr::Add(_, _) | Expr::Sub(_, _) => Prec::Add,
        Expr::Mul(_, _) | Expr::Div(_, _) => Prec::Mul,
        Expr::Pow(_, _) => Prec::Pow,
    }
}

fn maybe_paren(expr: &Expr, parent: Prec, style: Style) -> String {
    let s = render(expr, style);
    if prec(expr) < parent { format!("({s})") } else { s }
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) } else { format!("{n}") }
}
