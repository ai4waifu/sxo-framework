//! Symbolic differentiation.

use crate::expr::Expr;

/// Differentiate `expr` with respect to `var`.
pub fn differentiate(expr: &Expr, var: &str) -> Expr {
    match expr {
        Expr::Num(_) => Expr::num(0.0),
        Expr::Var(name) if name == var => Expr::num(1.0),
        Expr::Var(_) => Expr::num(0.0),
        Expr::Neg(a) => Expr::neg(differentiate(a, var)),
        Expr::Add(a, b) => Expr::add(differentiate(a, var), differentiate(b, var)),
        Expr::Sub(a, b) => Expr::sub(differentiate(a, var), differentiate(b, var)),
        Expr::Mul(a, b) => {
            // product rule
            Expr::add(Expr::mul(differentiate(a, var), (**b).clone()), Expr::mul((**a).clone(), differentiate(b, var)))
        }
        Expr::Div(a, b) => {
            // (a'b - ab') / b^2
            let num =
                Expr::sub(Expr::mul(differentiate(a, var), (**b).clone()), Expr::mul((**a).clone(), differentiate(b, var)));
            Expr::div(num, Expr::pow((**b).clone(), Expr::num(2.0)))
        }
        Expr::Pow(base, exp) => {
            // n * x^(n-1) * x' when exp is constant
            if let Expr::Num(n) = exp.as_ref() {
                let coef = Expr::num(*n);
                let new_exp = Expr::num(n - 1.0);
                Expr::mul(Expr::mul(coef, Expr::pow((**base).clone(), new_exp)), differentiate(base, var))
            }
            else {
                // fallback: leave as opaque zero for S0 non-constant exponents
                Expr::num(0.0)
            }
        }
        Expr::Sin(a) => Expr::mul(Expr::cos((**a).clone()), differentiate(a, var)),
        Expr::Cos(a) => Expr::mul(Expr::neg(Expr::sin((**a).clone())), differentiate(a, var)),
    }
}
