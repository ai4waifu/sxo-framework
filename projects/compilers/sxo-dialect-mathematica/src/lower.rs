//! Lower Mathematica Form ([`WExpr`]) into a session arena (`TermId`).

use athena::{
    AtomKind, Session, TermId, TermKind, clone_number, push_app_named, push_bool, push_list,
    push_null, push_symbol_name,
};

use crate::form::{WAtom, WExpr};

/// Structural `WExpr` → session arena `TermId`.
pub fn lower_wexpr(session: &mut Session, w: &WExpr) -> TermId {
    match w {
        WExpr::Atom(a) => match a {
            WAtom::Number(n) => {
                let span = athena::SourceSpan::default();
                session.arena.push(TermKind::Atom(AtomKind::Number(clone_number(n))), span)
            }
            WAtom::String(s) => {
                let span = athena::SourceSpan::default();
                session.arena.push(TermKind::Atom(AtomKind::String(s.clone())), span)
            }
            WAtom::Symbol(s) if s == "True" => push_bool(session, true),
            WAtom::Symbol(s) if s == "False" => push_bool(session, false),
            WAtom::Symbol(s) if s == "Null" => push_null(session),
            WAtom::Symbol(s) => push_symbol_name(session, s),
        },
        WExpr::List(items) => {
            let ids: Vec<TermId> = items.iter().map(|i| lower_wexpr(session, i)).collect();
            push_list(session, ids)
        }
        WExpr::Call { head, args } => match head.as_ref() {
            WExpr::Atom(WAtom::Symbol(name)) => {
                let arg_ids: Vec<TermId> = args.iter().map(|a| lower_wexpr(session, a)).collect();
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

/// Session arena `TermId` → structural `WExpr`.
pub fn wexpr_from_session(session: &Session, id: TermId) -> WExpr {
    match session.arena.get(id) {
        Some(TermKind::Atom(AtomKind::Number(n))) => WExpr::Atom(WAtom::Number(clone_number(n))),
        Some(TermKind::Atom(AtomKind::String(s))) => WExpr::Atom(WAtom::String(s.clone())),
        Some(TermKind::Atom(AtomKind::Boolean(true))) => WExpr::Atom(WAtom::Symbol("True".into())),
        Some(TermKind::Atom(AtomKind::Boolean(false))) => WExpr::Atom(WAtom::Symbol("False".into())),
        Some(TermKind::Atom(AtomKind::Null)) => WExpr::Atom(WAtom::Symbol("Null".into())),
        Some(TermKind::Atom(AtomKind::Symbol(sym))) => {
            let name = session.arena.symbols().resolve(*sym).unwrap_or("").to_string();
            WExpr::Atom(WAtom::Symbol(name))
        }
        Some(TermKind::List(items)) => {
            WExpr::List(items.iter().map(|i| wexpr_from_session(session, *i)).collect())
        }
        Some(TermKind::App { op, args }) => {
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
        None => WExpr::Atom(WAtom::Symbol(format!("TermId({})", id.0))),
    }
}
