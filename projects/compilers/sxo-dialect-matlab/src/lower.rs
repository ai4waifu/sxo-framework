//! MATLAB Form (session [`TermId`]) → neutral [`AthenaRequest`] (Living `27`).
//!
//! CST → term conversion lives in [`crate::parse`]. This module lifts dialect-shaped
//! applications into Session / Control contracts without inventing a second Form type.

use athena::{
    api::{AthenaRequest, ControlPlan, SessionCommand},
    ir::{Atom, TermNode},
    runtime::values::arena::{application_arguments, application_head_name, push_application_named},
    types::{BindingEvaluationPolicy, BindingKind, SymbolId, TermId},
    Session,
};

/// Lift a MATLAB-lowered term into a neutral [`AthenaRequest`].
pub fn lower_request(session: &mut Session, term: TermId) -> AthenaRequest {
    match application_head_name(session, term).as_deref() {
        Some("Set") => {
            if let Some(args) = application_arguments(session, term) {
                if let [lhs, rhs] = args.as_slice() {
                    if let Some(symbol) = symbol_atom(session, *lhs) {
                        return AthenaRequest::Command(SessionCommand::Define {
                            symbol,
                            value: *rhs,
                            kind: BindingKind::Session,
                            evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                        });
                    }
                }
            }
        }
        Some("CompoundExpression") => {
            if let Some(args) = application_arguments(session, term) {
                let steps: Vec<AthenaRequest> = args.iter().map(|t| lower_request(session, *t)).collect();
                return AthenaRequest::Control(ControlPlan::Sequence { steps });
            }
        }
        Some("If") | Some("Branch") => {
            if let Some(args) = application_arguments(session, term) {
                match args.as_slice() {
                    [cond, then_branch] => {
                        return AthenaRequest::Control(ControlPlan::Branch {
                            condition: *cond,
                            then_branch: Box::new(lower_request(session, *then_branch)),
                            else_branch: None,
                        });
                    }
                    [cond, then_branch, else_branch] => {
                        return AthenaRequest::Control(ControlPlan::Branch {
                            condition: *cond,
                            then_branch: Box::new(lower_request(session, *then_branch)),
                            else_branch: Some(Box::new(lower_request(session, *else_branch))),
                        });
                    }
                    _ => {}
                }
            }
        }
        Some("While") | Some("LoopWhile") => {
            if let Some(args) = application_arguments(session, term) {
                if let [cond, body] = args.as_slice() {
                    return AthenaRequest::Control(ControlPlan::LoopWhile {
                        condition: *cond,
                        body: Box::new(lower_request(session, *body)),
                    });
                }
            }
        }
        Some("Span") => {
            if let Some(args) = application_arguments(session, term) {
                let rewritten = push_application_named(session, "Range", args);
                return AthenaRequest::Term(rewritten);
            }
        }
        _ => {}
    }
    AthenaRequest::Term(term)
}

fn symbol_atom(session: &Session, term: TermId) -> Option<SymbolId> {
    match session.arena.get(term) {
        Some(TermNode::Atom(Atom::Symbol(symbol))) => Some(*symbol),
        _ => None,
    }
}
