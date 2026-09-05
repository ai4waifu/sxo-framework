//! MATLAB Form (session [`TermId`]) → neutral [`AthenaRequest`] (Living `27`).
//!
//! CST → term conversion lives in [`crate::parse`]. This module lifts dialect-shaped
//! applications into Session / Control contracts without inventing a second Form type.

use athena::{
    api::{AthenaRequest, ControlPlan, DomainGoal, SessionCommand},
    domains::{
        calculus::{CalculusRequest, DerivativeOrder},
        DomainRequest,
    },
    ir::{Atom, SemanticOperator, TermNode},
    runtime::values::arena::{application_arguments, number_from_id, push_semantic, symbol_name},
    types::{
        AssumptionSet, BindingEvaluationPolicy, BindingKind, IndexSpec, IntegerIndex, IntegerOffset, SymbolId,
        TermId,
    },
    Session,
};

use crate::surface::application_surface_name;

/// Lift a MATLAB-lowered term into a neutral [`AthenaRequest`].
pub fn lower_request(session: &mut Session, term: TermId) -> AthenaRequest {
    match application_surface_name(session, term).as_deref() {
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
        Some("For") | Some("CountedLoop") => {
            if let Some(args) = application_arguments(session, term) {
                if let [variable, iterator, body] = args.as_slice() {
                    return AthenaRequest::Control(ControlPlan::CountedLoop {
                        variable: *variable,
                        iterator: *iterator,
                        body: Box::new(lower_request(session, *body)),
                    });
                }
            }
        }
        Some("Part") => {
            if let Some(args) = application_arguments(session, term) {
                if args.len() >= 2 {
                    if let Some(axes) = args[1..].iter().map(|a| index_spec_of(session, *a)).collect::<Option<Vec<_>>>() {
                        return AthenaRequest::Control(ControlPlan::Index {
                            target: args[0],
                            axes,
                        });
                    }
                }
            }
        }
        Some("Span") => {
            if let Some(args) = application_arguments(session, term) {
                let rewritten = push_semantic(session, SemanticOperator::Range, args);
                return AthenaRequest::Term(rewritten);
            }
        }
        Some("diff") | Some("Diff") => {
            if let Some(args) = application_arguments(session, term) {
                match args.as_slice() {
                    [expr, var] => {
                        if let Some(variable) = symbol_atom(session, *var) {
                            return calculus_goal(CalculusRequest::Derivative {
                                expression: *expr,
                                variable,
                                order: DerivativeOrder::First,
                                assumptions: AssumptionSet::empty(),
                            });
                        }
                    }
                    [expr, var, order] => {
                        if let Some(variable) = symbol_atom(session, *var) {
                            if let Some(n) = number_from_id(session, *order).and_then(|n| n.as_exact_integer()) {
                                if n > 0 {
                                    let order = if n == 1 {
                                        DerivativeOrder::First
                                    } else {
                                        DerivativeOrder::Repeated(n as u32)
                                    };
                                    return calculus_goal(CalculusRequest::Derivative {
                                        expression: *expr,
                                        variable,
                                        order,
                                        assumptions: AssumptionSet::empty(),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Some("int") | Some("Int") | Some("integral") => {
            if let Some(args) = application_arguments(session, term) {
                if let [expr, var] = args.as_slice() {
                    if let Some(variable) = symbol_atom(session, *var) {
                        return calculus_goal(CalculusRequest::Integral {
                            expression: *expr,
                            variable,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
            }
        }
        _ => {}
    }
    AthenaRequest::Term(term)
}

fn calculus_goal(request: CalculusRequest) -> AthenaRequest {
    AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::Calculus(request)))
}

fn symbol_atom(session: &Session, term: TermId) -> Option<SymbolId> {
    match session.arena.get(term) {
        Some(TermNode::Atom(Atom::Symbol(symbol))) => Some(*symbol),
        _ => None,
    }
}

fn index_spec_of(session: &Session, term: TermId) -> Option<IndexSpec> {
    if let Some(n) = number_from_id(session, term).and_then(|n| n.as_exact_integer()) {
        return Some(IndexSpec::Scalar(IntegerIndex(n)));
    }
    match symbol_name(session, term).as_deref() {
        Some("All") | Some(":") => return Some(IndexSpec::All),
        Some("end") => return Some(IndexSpec::EndRelative(IntegerOffset(0))),
        _ => {}
    }
    match application_surface_name(session, term).as_deref() {
        Some("Range") | Some("Span") => {
            let args = application_arguments(session, term)?;
            match args.as_slice() {
                [start, end] => Some(IndexSpec::Range {
                    start: IntegerIndex(number_from_id(session, *start)?.as_exact_integer()?),
                    end: IntegerIndex(number_from_id(session, *end)?.as_exact_integer()?),
                    step: 1,
                }),
                [start, end, step] => Some(IndexSpec::Range {
                    start: IntegerIndex(number_from_id(session, *start)?.as_exact_integer()?),
                    end: IntegerIndex(number_from_id(session, *end)?.as_exact_integer()?),
                    step: number_from_id(session, *step)?.as_exact_integer()?,
                }),
                _ => None,
            }
        }
        // `end+k` may lower as Semantic Add or legacy Plus surface.
        Some("Plus") | Some("Add") => {
            let args = application_arguments(session, term)?;
            if args.len() == 2 && symbol_name(session, args[0]).as_deref() == Some("end") {
                let off = number_from_id(session, args[1])?.as_exact_integer()?;
                return Some(IndexSpec::EndRelative(IntegerOffset(off)));
            }
            None
        }
        _ => None,
    }
}
