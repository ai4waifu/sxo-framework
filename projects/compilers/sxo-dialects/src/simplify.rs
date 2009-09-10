//! Rule-based simplification (legacy flat `Expr` pipeline; no egg).

use crate::expr::Expr;

/// Bottom-up rewrite until a fixed point (bounded iterations).
pub fn simplify(expr: &Expr) -> Expr {
    let mut cur = expr.clone();
    for _ in 0..32 {
        let next = rewrite_once(&cur);
        if next == cur {
            break;
        }
        cur = next;
    }
    cur
}

fn rewrite_once(expr: &Expr) -> Expr {
    let rewritten = match expr {
        Expr::Neg(a) => Expr::neg(rewrite_once(a)),
        Expr::Add(a, b) => Expr::add(rewrite_once(a), rewrite_once(b)),
        Expr::Sub(a, b) => Expr::sub(rewrite_once(a), rewrite_once(b)),
        Expr::Mul(a, b) => Expr::mul(rewrite_once(a), rewrite_once(b)),
        Expr::Div(a, b) => Expr::div(rewrite_once(a), rewrite_once(b)),
        Expr::Pow(a, b) => Expr::pow(rewrite_once(a), rewrite_once(b)),
        Expr::Sin(a) => Expr::sin(rewrite_once(a)),
        Expr::Cos(a) => Expr::cos(rewrite_once(a)),
        other => other.clone(),
    };
    apply_local(&rewritten)
}

fn apply_local(expr: &Expr) -> Expr {
    // Pythagorean identity before generic add folding.
    if let Expr::Add(a, b) = expr {
        if is_sin_sq(a) && is_cos_sq(b) && same_trig_arg(a, b) {
            return Expr::num(1.0);
        }
        if is_cos_sq(a) && is_sin_sq(b) && same_trig_arg(a, b) {
            return Expr::num(1.0);
        }
    }

    match expr {
        Expr::Add(a, b) => match (a.as_ref(), b.as_ref()) {
            (Expr::Num(x), Expr::Num(y)) => Expr::num(x + y),
            (x, y) if y.is_zero() => x.clone(),
            (x, y) if x.is_zero() => y.clone(),
            _ => expr.clone(),
        },
        Expr::Sub(a, b) => match (a.as_ref(), b.as_ref()) {
            (Expr::Num(x), Expr::Num(y)) => Expr::num(x - y),
            (x, y) if y.is_zero() => x.clone(),
            _ => expr.clone(),
        },
        Expr::Mul(a, b) => match (a.as_ref(), b.as_ref()) {
            (Expr::Num(x), Expr::Num(y)) => Expr::num(x * y),
            (x, y) if x.is_zero() || y.is_zero() => Expr::num(0.0),
            (x, y) if y.is_one() => x.clone(),
            (x, y) if x.is_one() => y.clone(),
            _ => expr.clone(),
        },
        Expr::Div(a, b) => match (a.as_ref(), b.as_ref()) {
            (Expr::Num(x), Expr::Num(y)) if *y != 0.0 => Expr::num(x / y),
            (x, y) if y.is_one() => x.clone(),
            (x, _) if x.is_zero() => Expr::num(0.0),
            _ => expr.clone(),
        },
        Expr::Pow(a, b) => match (a.as_ref(), b.as_ref()) {
            (Expr::Num(x), Expr::Num(y)) => Expr::num(x.powf(*y)),
            (_, y) if y.is_zero() => Expr::num(1.0),
            (x, y) if y.is_one() => x.clone(),
            _ => expr.clone(),
        },
        Expr::Neg(a) => match a.as_ref() {
            Expr::Num(n) => Expr::num(-n),
            Expr::Neg(inner) => (**inner).clone(),
            _ => expr.clone(),
        },
        Expr::Sin(a) if a.is_zero() => Expr::num(0.0),
        Expr::Cos(a) if a.is_zero() => Expr::num(1.0),
        _ => expr.clone(),
    }
}

fn is_sin_sq(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Pow(base, exp)
            if matches!(base.as_ref(), Expr::Sin(_))
                && matches!(exp.as_ref(), Expr::Num(n) if *n == 2.0)
    )
}

fn is_cos_sq(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Pow(base, exp)
            if matches!(base.as_ref(), Expr::Cos(_))
                && matches!(exp.as_ref(), Expr::Num(n) if *n == 2.0)
    )
}

fn same_trig_arg(a: &Expr, b: &Expr) -> bool {
    fn arg_owned(e: &Expr) -> Option<Expr> {
        match e {
            Expr::Pow(base, _) => match base.as_ref() {
                Expr::Sin(x) | Expr::Cos(x) => Some((**x).clone()),
                _ => None,
            },
            _ => None,
        }
    }
    match (arg_owned(a), arg_owned(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}
