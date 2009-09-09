//! Bridge legacy flat [`Expr`] ↔ engine [`Term`] (Simple Math off-route helpers).
//!
//! [`Expr::Num`] is a **legacy frontend** form; lowering to kernel uses explicit machine-real
//! conversion only — not exact semantics.

#![allow(dead_code)]

use crate::{expr::Expr, term::Atom, term::Term};
use euler::Number;
use sxo_types::SxoError;

/// Lower a shared-subset [`Term`] into flat [`Expr`] (lossy for exact numbers).
pub fn term_to_expr(t: &Term) -> Result<Expr, SxoError> {
    match t {
        Term::Atom(Atom::Number(n)) => Ok(Expr::num(
            n.to_f64_lossy().ok_or_else(|| SxoError::new("bridge: number out of f64 range"))?,
        )),
        Term::Atom(Atom::Symbol(s)) => Ok(Expr::var(s.clone())),
        Term::Atom(Atom::String(_)) => Err(SxoError::new("bridge: strings not in Expr")),
        Term::List(_) => Err(SxoError::new("bridge: List not in Expr")),
        Term::App { head, args } => {
            let h = head.head_name().ok_or_else(|| SxoError::new("bridge: non-symbol head"))?;
            match h {
                "Plus" if args.len() == 2 => Ok(Expr::add(term_to_expr(&args[0])?, term_to_expr(&args[1])?)),
                "Plus" if args.len() > 2 => {
                    let mut acc = term_to_expr(&args[0])?;
                    for a in &args[1..] {
                        acc = Expr::add(acc, term_to_expr(a)?);
                    }
                    Ok(acc)
                }
                "Times" if args.len() == 2 => {
                    if args[0].is_neg_one() {
                        Ok(Expr::neg(term_to_expr(&args[1])?))
                    }
                    else {
                        Ok(Expr::mul(term_to_expr(&args[0])?, term_to_expr(&args[1])?))
                    }
                }
                "Times" if args.len() > 2 => {
                    let mut acc = term_to_expr(&args[0])?;
                    for a in &args[1..] {
                        acc = Expr::mul(acc, term_to_expr(a)?);
                    }
                    Ok(acc)
                }
                "Subtract" if args.len() == 2 => Ok(Expr::sub(term_to_expr(&args[0])?, term_to_expr(&args[1])?)),
                "Divide" if args.len() == 2 => Ok(Expr::div(term_to_expr(&args[0])?, term_to_expr(&args[1])?)),
                "Power" if args.len() == 2 => Ok(Expr::pow(term_to_expr(&args[0])?, term_to_expr(&args[1])?)),
                "Sin" if args.len() == 1 => Ok(Expr::sin(term_to_expr(&args[0])?)),
                "Cos" if args.len() == 1 => Ok(Expr::cos(term_to_expr(&args[0])?)),
                other => Err(SxoError::new(format!("bridge: unsupported head `{other}`"))),
            }
        }
    }
}

/// Lift flat [`Expr`] into engine [`Term`] (numbers become machine reals).
pub fn expr_to_term(e: &Expr) -> Term {
    match e {
        Expr::Num(n) => Term::number(Number::machine(*n)),
        Expr::Var(v) => Term::symbol(v.clone()),
        Expr::Neg(a) => Term::app("Times", vec![Term::int(-1), expr_to_term(a)]),
        Expr::Add(a, b) => Term::app("Plus", vec![expr_to_term(a), expr_to_term(b)]),
        Expr::Sub(a, b) => Term::app("Subtract", vec![expr_to_term(a), expr_to_term(b)]),
        Expr::Mul(a, b) => Term::app("Times", vec![expr_to_term(a), expr_to_term(b)]),
        Expr::Div(a, b) => Term::app("Divide", vec![expr_to_term(a), expr_to_term(b)]),
        Expr::Pow(a, b) => Term::app("Power", vec![expr_to_term(a), expr_to_term(b)]),
        Expr::Sin(a) => Term::app("Sin", vec![expr_to_term(a)]),
        Expr::Cos(a) => Term::app("Cos", vec![expr_to_term(a)]),
    }
}
