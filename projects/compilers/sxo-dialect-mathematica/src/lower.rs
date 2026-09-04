//! Lower Mathematica Form ([`WExpr`]) into a session arena (`ExprId`).

use athena::{
    ir::Atom,
    Session,
    types::ExprId,
    ir::ExprNode,
    runtime::values::numeric_clone::clone_number,
    runtime::values::arena::push_app_named,
    runtime::values::arena::push_bool,
    runtime::values::arena::push_list,
    runtime::values::arena::push_null,
    runtime::values::arena::push_symbol_name,
};

use crate::form::{WAtom, WExpr};

/// Structural `WExpr` → session arena `ExprId`.
pub fn lower_wexpr(session: &mut Session, w: &WExpr) -> ExprId {
    match w {
        WExpr::Atom(a) => match a {
            WAtom::Number(n) => {
                let span = athena::types::SourceSpan::default();
                session.arena.push(ExprNode::Atom(Atom::Number(clone_number(n))), span)
            }
            WAtom::String(s) => {
                let span = athena::types::SourceSpan::default();
                session.arena.push(ExprNode::Atom(Atom::String(s.clone())), span)
            }
            WAtom::Symbol(s) if s == "True" => push_bool(session, true),
            WAtom::Symbol(s) if s == "False" => push_bool(session, false),
            WAtom::Symbol(s) if s == "Null" => push_null(session),
            WAtom::Symbol(s) => push_symbol_name(session, s),
        },
        WExpr::List(items) => {
            let ids: Vec<ExprId> = items.iter().map(|i| lower_wexpr(session, i)).collect();
            push_list(session, ids)
        }
        WExpr::Call { head, args } => match head.as_ref() {
            WExpr::Atom(WAtom::Symbol(name)) => {
                let arg_ids: Vec<ExprId> = args.iter().map(|a| lower_wexpr(session, a)).collect();
                push_app_named(session, name, arg_ids)
            }
            other => {
                // Non-symbol head → `Application[head, args…]` for Athena `EvalDynamic`.
                let h = lower_wexpr(session, other);
                let mut wrapped = vec![h];
                wrapped.extend(args.iter().map(|a| lower_wexpr(session, a)));
                push_app_named(session, "Application", wrapped)
            }
        },
    }
}

/// Session arena `ExprId` → structural `WExpr`.
pub fn wexpr_from_session(session: &Session, id: ExprId) -> WExpr {
    match session.arena.get(id) {
        Some(ExprNode::Atom(Atom::Number(n))) => WExpr::Atom(WAtom::Number(clone_number(n))),
        Some(ExprNode::Atom(Atom::String(s))) => WExpr::Atom(WAtom::String(s.clone())),
        Some(ExprNode::Atom(Atom::Boolean(true))) => WExpr::Atom(WAtom::Symbol("True".into())),
        Some(ExprNode::Atom(Atom::Boolean(false))) => WExpr::Atom(WAtom::Symbol("False".into())),
        Some(ExprNode::Atom(Atom::Null)) => WExpr::Atom(WAtom::Symbol("Null".into())),
        Some(ExprNode::Atom(Atom::Symbol(sym))) => {
            let name = session.arena.symbols().resolve(*sym).unwrap_or("").to_string();
            WExpr::Atom(WAtom::Symbol(name))
        }
        Some(ExprNode::List(items)) => {
            WExpr::List(items.iter().map(|i| wexpr_from_session(session, *i)).collect())
        }
        Some(ExprNode::App { op, args }) => {
            let head_name = session.operators.name(*op).unwrap_or("?").to_string();
            if head_name == "Application" && !args.is_empty() {
                let head = wexpr_from_session(session, args[0]);
                let call_args: Vec<WExpr> = args[1..].iter().map(|a| wexpr_from_session(session, *a)).collect();
                return WExpr::Call { head: Box::new(head), args: call_args };
            }
            WExpr::Call {
                head: Box::new(WExpr::Atom(WAtom::Symbol(head_name))),
                args: args.iter().map(|a| wexpr_from_session(session, *a)).collect(),
            }
        }
        None => WExpr::Atom(WAtom::Symbol(format!("ExprId({})", id.0))),
    }
}
