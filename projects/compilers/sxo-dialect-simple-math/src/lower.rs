//! Bridge flat [`Expr`] ↔ session arena (`TermId`).
//!
//! [`Expr::Num`] is a **legacy frontend** form; lowering to kernel uses explicit machine-real
//! conversion only — not exact semantics.
//!
//! Living `27`: emit [`SemanticOperator`] / [`UnaryFunction`] only — never Mathematica surface names.

#![allow(dead_code)]

use athena::{
    ir::{ApplicationHead, Atom, SemanticOperator, TermNode, UnaryFunction},
    numeric::{Number, to_f64_lossy},
    runtime::values::arena::{application_arguments, number_from_id, push_semantic, push_symbol_name, symbol_name},
    Session,
    types::{SourceSpan, TermId},
};
use sxo_types::SxoError;

use crate::form::Expr;

/// Lower a shared-subset arena node into flat [`Expr`] (lossy for exact numbers).
pub fn expr_from_session(session: &Session, id: TermId) -> Result<Expr, SxoError> {
    match session.arena.get(id) {
        Some(TermNode::Atom(Atom::Number(n))) => {
            Ok(Expr::num(to_f64_lossy(n).ok_or_else(|| SxoError::new("bridge: number out of f64 range"))?))
        }
        Some(TermNode::Atom(Atom::Symbol(_))) => Ok(Expr::var(symbol_name(session, id).unwrap_or_default())),
        Some(TermNode::Atom(Atom::Boolean(true))) => Ok(Expr::var("True")),
        Some(TermNode::Atom(Atom::Boolean(false))) => Ok(Expr::var("False")),
        Some(TermNode::Atom(Atom::Null)) => Ok(Expr::var("Null")),
        Some(TermNode::Atom(Atom::String(_))) => Err(SxoError::new("bridge: strings not in Expr")),
        Some(TermNode::Collection { .. }) => Err(SxoError::new("bridge: List not in Expr")),
        Some(TermNode::Application { head, .. }) => {
            let args = application_arguments(session, id).unwrap_or_default();
            match *head {
                ApplicationHead::Semantic(op) => expr_from_semantic(session, op, &args),
                ApplicationHead::Extension(_) => Err(SxoError::new("bridge: extension head not in Expr")),
            }
        }
        None => Err(SxoError::new(format!("bridge: missing TermId({})", id.0))),
    }
}

fn expr_from_semantic(session: &Session, op: SemanticOperator, args: &[TermId]) -> Result<Expr, SxoError> {
    match op {
        SemanticOperator::Add if args.len() == 2 => {
            Ok(Expr::add(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
        }
        SemanticOperator::Add if args.len() > 2 => {
            let mut acc = expr_from_session(session, args[0])?;
            for a in &args[1..] {
                acc = Expr::add(acc, expr_from_session(session, *a)?);
            }
            Ok(acc)
        }
        SemanticOperator::Multiply if args.len() == 2 => {
            if is_neg_one(session, args[0]) {
                Ok(Expr::neg(expr_from_session(session, args[1])?))
            } else {
                Ok(Expr::mul(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
            }
        }
        SemanticOperator::Multiply if args.len() > 2 => {
            let mut acc = expr_from_session(session, args[0])?;
            for a in &args[1..] {
                acc = Expr::mul(acc, expr_from_session(session, *a)?);
            }
            Ok(acc)
        }
        SemanticOperator::Negate if args.len() == 1 => Ok(Expr::neg(expr_from_session(session, args[0])?)),
        SemanticOperator::Subtract if args.len() == 2 => {
            Ok(Expr::sub(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
        }
        SemanticOperator::Divide if args.len() == 2 => {
            Ok(Expr::div(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
        }
        SemanticOperator::Power if args.len() == 2 => {
            Ok(Expr::pow(expr_from_session(session, args[0])?, expr_from_session(session, args[1])?))
        }
        SemanticOperator::Unary(UnaryFunction::Sin) if args.len() == 1 => {
            Ok(Expr::sin(expr_from_session(session, args[0])?))
        }
        SemanticOperator::Unary(UnaryFunction::Cos) if args.len() == 1 => {
            Ok(Expr::cos(expr_from_session(session, args[0])?))
        }
        other => Err(SxoError::new(format!("bridge: unsupported semantic `{}`", other.debug_label()))),
    }
}

/// Lift flat [`Expr`] into a session arena [`TermId`] (numbers become machine reals).
pub fn lower_expr(session: &mut Session, e: &Expr) -> TermId {
    match e {
        Expr::Num(n) => session
            .arena
            .push(TermNode::Atom(Atom::Number(Number::machine(*n))), SourceSpan::default()),
        Expr::Var(v) => push_symbol_name(session, v),
        Expr::Neg(a) => {
            let inner = lower_expr(session, a);
            push_semantic(session, SemanticOperator::Negate, vec![inner])
        }
        Expr::Add(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_semantic(session, SemanticOperator::Add, vec![left, right])
        }
        Expr::Sub(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_semantic(session, SemanticOperator::Subtract, vec![left, right])
        }
        Expr::Mul(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_semantic(session, SemanticOperator::Multiply, vec![left, right])
        }
        Expr::Div(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_semantic(session, SemanticOperator::Divide, vec![left, right])
        }
        Expr::Pow(a, b) => {
            let left = lower_expr(session, a);
            let right = lower_expr(session, b);
            push_semantic(session, SemanticOperator::Power, vec![left, right])
        }
        Expr::Sin(a) => {
            let inner = lower_expr(session, a);
            push_semantic(session, SemanticOperator::from_unary(UnaryFunction::Sin), vec![inner])
        }
        Expr::Cos(a) => {
            let inner = lower_expr(session, a);
            push_semantic(session, SemanticOperator::from_unary(UnaryFunction::Cos), vec![inner])
        }
    }
}

fn is_neg_one(session: &Session, id: TermId) -> bool {
    matches!(number_from_id(session, id), Some(n) if *n == Number::small_int(-1))
}
