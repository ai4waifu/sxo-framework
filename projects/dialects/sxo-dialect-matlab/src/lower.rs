//! MATLAB Form (session [`TermId`]) → neutral [`AthenaRequest`] (Living `14`).
//!
//! CST → term conversion lives in [`crate::parse`]. This module lifts dialect-shaped
//! applications into Session / Control contracts without inventing a second Form type.

use athena::{
    Session,
    api::{AthenaRequest, ControlPlan, DomainGoal, SessionCommand},
    domains::{
        DomainRequest,
        calculus::{CalculusRequest, DerivativeOrder},
        linear_algebra::MatrixValue,
    },
    ir::{Atom, SemanticOperator, TermNode},
    numeric::{Integer, Rational},
    runtime::values::{
        arena::{application_arguments, number_from_id, push_semantic, symbol_name},
        numeric_clone::{clone_integer, clone_rational},
    },
    types::{AssumptionSet, BindingEvaluationPolicy, BindingKind, IndexSpec, IntegerIndex, IntegerOffset, SymbolId, TermId},
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
                    let body_req = lower_request(session, *body);
                    if matches!(body_req, AthenaRequest::Term(_)) {
                        return AthenaRequest::Control(ControlPlan::CountedLoop {
                            variable: *variable,
                            iterator: *iterator,
                            body: Box::new(body_req),
                        });
                    }
                    // Athena CountedLoop only unrolls `Term` bodies. Assignment / Sequence
                    // bodies expand here into Define(var) + body steps.
                    if let (Some(symbol), Some(items)) =
                        (symbol_atom(session, *variable), expand_counted_iterator(session, *iterator))
                    {
                        let mut steps = Vec::with_capacity(items.len().saturating_mul(2));
                        for item in items {
                            steps.push(AthenaRequest::Command(SessionCommand::Define {
                                symbol,
                                value: item,
                                kind: BindingKind::Session,
                                evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                            }));
                            steps.push(lower_request(session, *body));
                        }
                        return AthenaRequest::Control(ControlPlan::Sequence { steps });
                    }
                    return AthenaRequest::Control(ControlPlan::CountedLoop {
                        variable: *variable,
                        iterator: *iterator,
                        body: Box::new(body_req),
                    });
                }
            }
        }
        Some("Try") | Some("Recover") => {
            if let Some(args) = application_arguments(session, term) {
                if let [body, handler] = args.as_slice() {
                    return AthenaRequest::Control(ControlPlan::Recover {
                        body: Box::new(lower_request(session, *body)),
                        handler: Box::new(lower_request(session, *handler)),
                    });
                }
            }
        }
        Some("error") | Some("Error") | Some("Reject") => {
            return AthenaRequest::Control(ControlPlan::Reject);
        }
        Some("LinearSolve") | Some("Mldivide") => {
            if let Some(args) = application_arguments(session, term) {
                if let [a_term, b_term] = args.as_slice() {
                    if let (Some(a_mat), Some(b_mat)) =
                        (matrix_from_nested_list(session, *a_term), matrix_from_nested_list(session, *b_term))
                    {
                        let a = session.matrix_objects.intern(a_mat);
                        let b = session.matrix_objects.intern(b_mat);
                        return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                            athena::domains::linear_algebra::LinearAlgebraRequest::Solve { a, b },
                        )));
                    }
                }
            }
        }
        Some("Part") => {
            if let Some(args) = application_arguments(session, term) {
                if args.len() >= 2 {
                    if let Some(axes) = args[1..].iter().map(|a| index_spec_of(session, *a)).collect::<Option<Vec<_>>>() {
                        return AthenaRequest::Control(ControlPlan::Index { target: args[0], axes });
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
        // Parse maps `diff` → Extension `D`, `int` → Extension `Integrate`.
        Some("diff") | Some("Diff") | Some("D") => {
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
                                    let order =
                                        if n == 1 { DerivativeOrder::First } else { DerivativeOrder::Repeated(n as u32) };
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
        Some("int") | Some("Int") | Some("integral") | Some("Integrate") => {
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

fn expand_counted_iterator(session: &mut Session, iterator: TermId) -> Option<Vec<TermId>> {
    if let Some(TermNode::Collection { elements, .. }) = session.arena.get(iterator) {
        return Some(elements.clone());
    }
    let args = application_arguments(session, iterator)?;
    let ints: Option<Vec<i64>> = args.iter().map(|t| number_from_id(session, *t).and_then(|n| n.as_exact_integer())).collect();
    let ints = ints?;
    let values = match ints.as_slice() {
        [a, b] => expand_span(*a, 1, *b),
        // Athena `Range[start, end, step]` argument order.
        [a, b, step] => expand_span(*a, *step, *b),
        _ => return None,
    }?;
    Some(values.into_iter().map(|v| session.builder().int(v, Default::default())).collect())
}

fn expand_span(start: i64, step: i64, end: i64) -> Option<Vec<i64>> {
    if step == 0 {
        return None;
    }
    let mut out = Vec::new();
    let mut cur = start;
    if step > 0 {
        while cur <= end {
            out.push(cur);
            cur = cur.checked_add(step)?;
        }
    }
    else {
        while cur >= end {
            out.push(cur);
            cur = cur.checked_add(step)?;
        }
    }
    Some(out)
}

fn term_scalar_rational(session: &Session, term: TermId) -> Option<Rational> {
    let n = number_from_id(session, term)?;
    if let Some(i) = n.as_exact_integer() {
        return Some(Rational::new(Integer::from_i64(i), Integer::one()));
    }
    if let Some(i) = n.as_integer() {
        return Some(Rational::from_integer(clone_integer(i)));
    }
    n.as_rational().map(clone_rational)
}

fn matrix_from_nested_list(session: &Session, term: TermId) -> Option<MatrixValue> {
    match session.arena.get(term) {
        Some(TermNode::Collection { elements: rows, .. }) if !rows.is_empty() => {
            if matches!(session.arena.get(rows[0]), Some(TermNode::Collection { .. })) {
                let mut data = Vec::new();
                let mut cols: Option<u64> = None;
                for row in rows {
                    let cells = match session.arena.get(*row) {
                        Some(TermNode::Collection { elements: cells, .. }) => cells.clone(),
                        _ => return None,
                    };
                    let c = cells.len() as u64;
                    match cols {
                        Some(prev) if prev != c => return None,
                        None => cols = Some(c),
                        _ => {}
                    }
                    for cell in cells {
                        data.push(term_scalar_rational(session, cell)?);
                    }
                }
                MatrixValue::from_rationals_row_major(rows.len() as u64, cols.unwrap_or(0), data).ok()
            }
            else {
                let mut data = Vec::with_capacity(rows.len());
                for cell in rows {
                    data.push(term_scalar_rational(session, *cell)?);
                }
                MatrixValue::from_rationals_row_major(1, data.len() as u64, data).ok()
            }
        }
        _ => {
            let r = term_scalar_rational(session, term)?;
            MatrixValue::from_rationals_row_major(1, 1, vec![r]).ok()
        }
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
