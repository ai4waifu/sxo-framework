//! Bridge flat [`Expr`] ↔ session arena (`TermId`).
//!
//! [`Expr::Num`] is a **legacy frontend** form; lowering to kernel uses explicit machine-real
//! conversion only — not exact semantics.

#![allow(dead_code)]

use athena::{
    ir::Atom,
    numeric::Number,
    Session,
    types::SourceSpan,
    types::TermId,
    ir::TermNode,
    runtime::values::arena::application_arguments,
    runtime::values::arena::application_head_name,
    runtime::values::arena::number_from_id,
    numeric::to_f64_lossy,
    runtime::values::arena::push_application_named,
    runtime::values::arena::push_int,
    runtime::values::arena::push_symbol_name,
    runtime::values::arena::symbol_name,
};
use sxo_types::SxoError;

use crate::form::Expr;

/// Lower a shared-subset arena node into flat [`Expr`] (lossy for exact numbers).
pub fn expr_from_session(session: &Session, id: TermId) -> Result<Expr, SxoError> {
    match session.arena.get(id) {
        Some(TermNode::Atom(Atom::Number(n))) => {
            Ok(Expr::num(to_f64_lossy(n).ok_or_else(|| SxoError::new("bridge: number out of f64 range"))?))
        }
        Some(TermNode::Atom(Atom::Symbol(_))) => {
            Ok(Expr::var(symbol_name(session, id).unwrap_or_default()))
        }
        Some(TermNode::Atom(Atom::Boolean(true))) => Ok(Expr::var("True")),
        Some(TermNode::Atom(Atom::Boolean(false))) => Ok(Expr::var("False")),
        Some(TermNode::Atom(Atom::Null)) => Ok(Expr::var("Null")),
        Some(TermNode::Atom(Atom::String(_))) => Err(SxoError::new("bridge: strings not in Expr")),
        Some(TermNode::Collection { .. }) => Err(SxoError::new("bridge: List not in Expr")),
        Some(TermNode::Application { .. }) => {
            let h = application_head_name(session, id).ok_or_else(|| SxoError::new("bridge: non-symbol head"))?;
            let args = application_arguments(session, id).unwrap_or_default();
            match h.as_str() {
                "Plus" if args.len() == 2 => Ok(Expr::add(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?)),
                "Plus" if args.len() > 2 => {
                    let mut acc = expr_from_session(session, args[0])?;
                    for a in &args[1..] {
                        acc = Expr::add(acc, expr_from_session(session, *a)?);
                    }
                    Ok(acc)
                }
                "Times" if args.len() == 2 => {
                    if is_neg_one(session, args[0]) {
                        Ok(Expr::neg(expr_from_session(session, args[1])?))
                    }
                    else {
                        Ok(Expr::mul(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
                    }
                }
                "Times" if args.len() > 2 => {
                    let mut acc = expr_from_session(session, args[0])?;
                    for a in &args[1..] {
                        acc = Expr::mul(acc, expr_from_session(session, *a)?);
                    }
                    Ok(acc)
                }
                "Subtract" if args.len() == 2 => {
                    Ok(Expr::sub(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
                }
                "Divide" if args.len() == 2 => {
                    Ok(Expr::div(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
                }
                "Power" if args.len() == 2 => {
                    Ok(Expr::pow(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
                }
                "Sin" if args.len() == 1 => Ok(Expr::sin(expr_from_session(session, args[0])?)),
                "Cos" if args.len() == 1 => Ok(Expr::cos(expr_from_session(session, args[0])?)),
                other => Err(SxoError::new(format!("bridge: unsupported head `{other}`"))),
            }
        }
        None => Err(SxoError::new(format!("bridge: missing TermId({})", id.0))),
    }
}

/// Lift flat [`Expr`] into a session arena [`TermId`] (numbers become machine reals).
pub fn lower_expr(session: &mut Session, e: &Expr) -> TermId {
    match e {
        Expr::Num(n) => {
            session
                .arena
                .push(TermNode::Atom(Atom::Number(Number::machine(*n))), SourceSpan::default())
        }
        Expr::Var(v) => push_symbol_name(session, v),
        Expr::Neg(a) => {
            let neg1 = push_int(session, -1);
            let inner = lower_expr(session, a);
            push_application_named(session, "Times", vec![neg1, inner])
        }
        Expr::Add(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_application_named(session, "Plus", vec![left, right])
        }
        Expr::Sub(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_application_named(session, "Subtract", vec![left, right])
        }
        Expr::Mul(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_application_named(session, "Times", vec![left, right])
        }
        Expr::Div(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_application_named(session, "Divide", vec![left, right])
        }
        Expr::Pow(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_application_named(session, "Power", vec![left, right])
        }
        Expr::Sin(a) => {
            let inner = lower_expr(session, a);
            push_application_named(session, "Sin", vec![inner])
        }
        Expr::Cos(a) => {
            let inner = lower_expr(session, a);
            push_application_named(session, "Cos", vec![inner])
        }
    }
}

fn is_neg_one(session: &Session, id: TermId) -> bool {
    matches!(number_from_id(session, id), Some(n) if *n == Number::small_int(-1))
}
