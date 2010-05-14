//! Bridge flat [`Expr`] ↔ session arena (`TermId`).
//!
//! [`Expr::Num`] is a **legacy frontend** form; lowering to kernel uses explicit machine-real
//! conversion only — not exact semantics.
//!
//! Living `05`/`14`: lowercase surface heads map to [`SemanticOperator`] / [`UnaryFunction`].
//! Never emit Mathematica CapCase as Simple Math truth.

#![allow(dead_code)]

use athena::{
    Session,
    ir::{ApplicationHead, Atom, SemanticOperator, TermNode, UnaryFunction},
    numeric::{Number, to_f64_lossy},
    runtime::values::arena::{
        application_arguments, number_from_id, push_extension, push_list, push_semantic, push_symbol_name, symbol_name,
    },
    types::{SourceSpan, TermId},
};
use sxo_types::SxoError;

use crate::form::{DictKey, Expr};

/// Lower a shared-subset arena node into flat [`Expr`] (lossy for exact numbers).
pub fn expr_from_session(session: &Session, id: TermId) -> Result<Expr, SxoError> {
    match session.arena.get(id) {
        Some(TermNode::Atom(Atom::Number(n))) => {
            Ok(Expr::num(to_f64_lossy(n).ok_or_else(|| SxoError::new("bridge: number out of f64 range"))?))
        }
        Some(TermNode::Atom(Atom::Symbol(_))) => Ok(Expr::var(symbol_name(session, id).unwrap_or_default())),
        Some(TermNode::Atom(Atom::Boolean(true))) => Ok(Expr::var("true")),
        Some(TermNode::Atom(Atom::Boolean(false))) => Ok(Expr::var("false")),
        Some(TermNode::Atom(Atom::Null)) => Ok(Expr::var("null")),
        Some(TermNode::Atom(Atom::Constant(c))) => Ok(Expr::var(c.debug_label().to_ascii_lowercase())),
        Some(TermNode::Atom(Atom::String(s))) => Ok(Expr::var(format!("\"{s}\""))),
        Some(TermNode::Collection { elements: items, .. }) => {
            let items = items.clone();
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(expr_from_session(session, item)?);
            }
            Ok(Expr::list(out))
        }
        Some(TermNode::Application { head, .. }) => {
            let args = application_arguments(session, id).unwrap_or_default();
            match *head {
                ApplicationHead::Semantic(op) => expr_from_semantic(session, op, &args),
                ApplicationHead::Extension(oid) => {
                    let name = session
                        .extensions
                        .display_name(oid)
                        .unwrap_or("unknown")
                        .to_ascii_lowercase();
                    if name == "dict" {
                        return dict_from_flat_pairs(session, &args);
                    }
                    let mut out = Vec::with_capacity(args.len());
                    for a in args {
                        out.push(expr_from_session(session, a)?);
                    }
                    Ok(Expr::call(name, out))
                }
            }
        }
        None => Err(SxoError::new(format!("bridge: missing TermId({})", id.0))),
    }
}

fn dict_from_flat_pairs(session: &Session, args: &[TermId]) -> Result<Expr, SxoError> {
    if args.len() % 2 != 0 {
        return Err(SxoError::new("bridge: dict extension expects flat key/value pairs"));
    }
    let mut entries = Vec::with_capacity(args.len() / 2);
    for chunk in args.chunks(2) {
        let key = dict_key_from_session(session, chunk[0])?;
        let value = expr_from_session(session, chunk[1])?;
        entries.push((key, value));
    }
    Ok(Expr::dict(entries))
}

fn dict_key_from_session(session: &Session, id: TermId) -> Result<DictKey, SxoError> {
    match session.arena.get(id) {
        Some(TermNode::Atom(Atom::Symbol(_))) => Ok(DictKey::Ident(symbol_name(session, id).unwrap_or_default())),
        Some(TermNode::Atom(Atom::String(s))) => Ok(DictKey::String(s.clone())),
        _ => Err(SxoError::new("bridge: dict key must be symbol or string")),
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
        SemanticOperator::Unary(UnaryFunction::Sin) if args.len() == 1 => Ok(Expr::sin(expr_from_session(session, args[0])?)),
        SemanticOperator::Unary(UnaryFunction::Cos) if args.len() == 1 => Ok(Expr::cos(expr_from_session(session, args[0])?)),
        other => {
            let mut out = Vec::with_capacity(args.len());
            for a in args {
                out.push(expr_from_session(session, *a)?);
            }
            Ok(Expr::call(other.debug_label().to_ascii_lowercase(), out))
        }
    }
}

/// Lift flat [`Expr`] into a session arena [`TermId`] (numbers become machine reals).
pub fn lower_expr(session: &mut Session, e: &Expr) -> TermId {
    match e {
        Expr::Num(n) => session.arena.push(TermNode::Atom(Atom::Number(Number::machine(*n))), SourceSpan::default()),
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
        Expr::Call { head, args } => {
            let ids: Vec<TermId> = args.iter().map(|a| lower_expr(session, a)).collect();
            match head.as_str() {
                "sin" => push_semantic(session, SemanticOperator::from_unary(UnaryFunction::Sin), ids),
                "cos" => push_semantic(session, SemanticOperator::from_unary(UnaryFunction::Cos), ids),
                other => {
                    let op = session.extensions.intern(other);
                    push_extension(session, op, ids)
                }
            }
        }
        Expr::List(items) => {
            let ids: Vec<TermId> = items.iter().map(|a| lower_expr(session, a)).collect();
            push_list(session, ids)
        }
        Expr::Dict(entries) => {
            let mut flat = Vec::with_capacity(entries.len() * 2);
            for (key, value) in entries {
                flat.push(lower_dict_key(session, key));
                flat.push(lower_expr(session, value));
            }
            let op = session.extensions.intern("dict");
            push_extension(session, op, flat)
        }
    }
}

fn lower_dict_key(session: &mut Session, key: &DictKey) -> TermId {
    match key {
        DictKey::Ident(s) => push_symbol_name(session, s),
        DictKey::String(s) => session.arena.push(TermNode::Atom(Atom::String(s.clone())), SourceSpan::default()),
    }
}

fn is_neg_one(session: &Session, id: TermId) -> bool {
    matches!(number_from_id(session, id), Some(n) if *n == Number::small_int(-1))
}
