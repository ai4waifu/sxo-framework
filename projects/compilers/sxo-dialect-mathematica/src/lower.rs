//! Lower Mathematica Form ([`WExpr`]) into Athena session terms / requests (Living `27`).

use athena::{
    api::{AthenaRequest, ControlPlan, SessionCommand},
    ir::{Atom, TermNode},
    runtime::values::arena::{
        push_application_named, push_bool, push_int, push_list, push_null, push_symbol_name,
    },
    runtime::values::numeric_clone::clone_number,
    types::{BindingEvaluationPolicy, BindingKind, SymbolId, TermId},
    Session,
};

use crate::form::{WAtom, WExpr};

/// Structural `WExpr` → session arena [`TermId`].
///
/// Prefer [`lower_request`] when the form carries session / control semantics.
pub fn lower_wexpr(session: &mut Session, w: &WExpr) -> TermId {
    match w {
        WExpr::Atom(a) => match a {
            WAtom::Number(n) => {
                let span = athena::types::SourceSpan::default();
                session.arena.push(TermNode::Atom(Atom::Number(clone_number(n))), span)
            }
            WAtom::String(s) => {
                let span = athena::types::SourceSpan::default();
                session.arena.push(TermNode::Atom(Atom::String(s.clone())), span)
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
            WExpr::Atom(WAtom::Symbol(name)) if name == "Span" => lower_span_as_range(session, args),
            WExpr::Atom(WAtom::Symbol(name)) => {
                let arg_ids: Vec<TermId> = args.iter().map(|a| lower_wexpr(session, a)).collect();
                push_application_named(session, name, arg_ids)
            }
            other => {
                let h = lower_wexpr(session, other);
                let mut wrapped = vec![h];
                wrapped.extend(args.iter().map(|a| lower_wexpr(session, a)));
                push_application_named(session, "Application", wrapped)
            }
        },
    }
}

/// Dialect Form → neutral [`AthenaRequest`].
///
/// Maps Mathematica surface assignment / iteration into Session / Control contracts.
/// Other forms lower to [`AthenaRequest::Term`].
pub fn lower_request(session: &mut Session, w: &WExpr) -> AthenaRequest {
    match w {
        WExpr::Call { head, args } => match head.as_ref() {
            WExpr::Atom(WAtom::Symbol(name)) => match (name.as_str(), args.as_slice()) {
                ("Set", [lhs, rhs]) => {
                    if let Some(symbol) = symbol_of(session, lhs) {
                        let value = lower_wexpr(session, rhs);
                        return AthenaRequest::Command(SessionCommand::Define {
                            symbol,
                            value,
                            kind: BindingKind::Session,
                            evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                        });
                    }
                }
                ("SetDelayed", [lhs, rhs]) => {
                    // Pattern lhs `f[…]` needs `RegisterRuleDispatch` in a later slice.
                    if matches!(lhs, WExpr::Atom(WAtom::Symbol(_))) {
                        if let Some(symbol) = symbol_of(session, lhs) {
                            let value = lower_wexpr(session, rhs);
                            return AthenaRequest::Command(SessionCommand::Define {
                                symbol,
                                value,
                                kind: BindingKind::Session,
                                evaluation: BindingEvaluationPolicy::StoreResidualTerm,
                            });
                        }
                    }
                }
                ("Table", [body, iter]) => {
                    let body_term = lower_wexpr(session, body);
                    let binder = extract_table_binder(session, iter).unwrap_or(body_term);
                    let range = normalize_table_range(session, iter);
                    return AthenaRequest::Control(ControlPlan::Iterate {
                        binder,
                        range,
                        body: Box::new(AthenaRequest::Term(body_term)),
                        evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                    });
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
    AthenaRequest::Term(lower_wexpr(session, w))
}

fn symbol_of(session: &mut Session, w: &WExpr) -> Option<SymbolId> {
    match w {
        WExpr::Atom(WAtom::Symbol(name)) => Some(session.arena.symbols_mut().intern(name)),
        _ => None,
    }
}

fn extract_table_binder(session: &mut Session, iter: &WExpr) -> Option<TermId> {
    match iter {
        WExpr::List(items) if !items.is_empty() => Some(lower_wexpr(session, &items[0])),
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "List")
                && !args.is_empty() =>
        {
            Some(lower_wexpr(session, &args[0]))
        }
        _ => None,
    }
}

/// `Table` iterator `{i, n}` / `{i, a, b}` → ordered collection of values (binder excluded).
fn normalize_table_range(session: &mut Session, iter: &WExpr) -> TermId {
    let items: Option<Vec<&WExpr>> = match iter {
        WExpr::List(items) => Some(items.iter().collect()),
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "List") =>
        {
            Some(args.iter().collect())
        }
        _ => None,
    };
    match items.as_deref() {
        Some([_binder, end]) => {
            if let Some(n) = exact_i64(end) {
                let vals: Vec<TermId> = (1..=n).map(|i| push_int(session, i)).collect();
                return push_list(session, vals);
            }
        }
        Some([_binder, start, end]) => {
            if let (Some(a), Some(b)) = (exact_i64(start), exact_i64(end)) {
                let vals: Vec<TermId> = (a..=b).map(|i| push_int(session, i)).collect();
                return push_list(session, vals);
            }
        }
        Some([_binder, start, end, step]) => {
            if let (Some(a), Some(b), Some(s)) = (exact_i64(start), exact_i64(end), exact_i64(step)) {
                if s != 0 {
                    let mut vals = Vec::new();
                    let mut cur = a;
                    if s > 0 {
                        while cur <= b {
                            vals.push(push_int(session, cur));
                            cur += s;
                        }
                    } else {
                        while cur >= b {
                            vals.push(push_int(session, cur));
                            cur += s;
                        }
                    }
                    return push_list(session, vals);
                }
            }
        }
        _ => {}
    }
    lower_wexpr(session, iter)
}

fn lower_span_as_range(session: &mut Session, args: &[WExpr]) -> TermId {
    let arg_ids: Vec<TermId> = args.iter().map(|a| lower_wexpr(session, a)).collect();
    push_application_named(session, "Range", arg_ids)
}

fn exact_i64(w: &WExpr) -> Option<i64> {
    match w {
        WExpr::Atom(WAtom::Number(n)) => n.as_exact_integer(),
        _ => None,
    }
}

/// Session arena [`TermId`] → structural `WExpr`.
pub fn wexpr_from_session(session: &Session, id: TermId) -> WExpr {
    match session.arena.get(id) {
        Some(TermNode::Atom(Atom::Number(n))) => WExpr::Atom(WAtom::Number(clone_number(n))),
        Some(TermNode::Atom(Atom::String(s))) => WExpr::Atom(WAtom::String(s.clone())),
        Some(TermNode::Atom(Atom::Boolean(true))) => WExpr::Atom(WAtom::Symbol("True".into())),
        Some(TermNode::Atom(Atom::Boolean(false))) => WExpr::Atom(WAtom::Symbol("False".into())),
        Some(TermNode::Atom(Atom::Null)) => WExpr::Atom(WAtom::Symbol("Null".into())),
        Some(TermNode::Atom(Atom::Symbol(sym))) => {
            let name = session.arena.symbols().resolve(*sym).unwrap_or("").to_string();
            WExpr::Atom(WAtom::Symbol(name))
        }
        Some(TermNode::Collection { elements: items, .. }) => {
            WExpr::List(items.iter().map(|i| wexpr_from_session(session, *i)).collect())
        }
        Some(TermNode::Application { head: op, arguments: args }) => {
            let head_name = session.operators.name(*op).unwrap_or("?").to_string();
            if head_name == "Application" && !args.is_empty() {
                let head = wexpr_from_session(session, args[0]);
                let call_args: Vec<WExpr> = args[1..].iter().map(|a| wexpr_from_session(session, *a)).collect();
                return WExpr::Call {
                    head: Box::new(head),
                    args: call_args,
                };
            }
            WExpr::Call {
                head: Box::new(WExpr::Atom(WAtom::Symbol(head_name))),
                args: args.iter().map(|a| wexpr_from_session(session, *a)).collect(),
            }
        }
        None => WExpr::Atom(WAtom::Symbol(format!("TermId({})", id.0))),
    }
}
