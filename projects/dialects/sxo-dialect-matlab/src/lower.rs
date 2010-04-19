//! MATLAB Form → session terms / neutral [`AthenaRequest`] (Living `14`).
//!
//! Formal entry: [`lower_request`] on [`MatlabForm`]. Hosts must keep Form on
//! parse objects; do not reconstruct Form from arena [`TermId`]s.

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
        arena::{
            application_arguments, number_from_id, push_bool, push_list, push_null, push_semantic, push_symbol_name,
            symbol_name,
        },
        numeric_clone::{clone_integer, clone_number, clone_rational},
    },
    types::{
        AssumptionSet, BindingEvaluationPolicy, BindingKind, IndexSpec, IntegerIndex, IntegerOffset, SourceSpan, SymbolId,
        TermId,
    },
};

use crate::{
    form::{MatlabAtom, MatlabForm},
    surface::{application_surface_name, push_matlab_call},
};

/// Materialize a [`MatlabForm`] into the session arena (transitional bridge).
pub fn form_to_term(session: &mut Session, form: &MatlabForm) -> TermId {
    match form {
        MatlabForm::Atom(MatlabAtom::Number(n)) => {
            session.arena.push(TermNode::Atom(Atom::Number(clone_number(n))), SourceSpan::default())
        }
        MatlabForm::Atom(MatlabAtom::String(s)) => {
            session.arena.push(TermNode::Atom(Atom::String(s.clone())), SourceSpan::default())
        }
        MatlabForm::Atom(MatlabAtom::Symbol(name)) => push_symbol_name(session, name),
        MatlabForm::Atom(MatlabAtom::Bool(b)) => push_bool(session, *b),
        MatlabForm::Atom(MatlabAtom::Null) => push_null(session),
        MatlabForm::List(items) => {
            let ids: Vec<TermId> = items.iter().map(|i| form_to_term(session, i)).collect();
            push_list(session, ids)
        }
        MatlabForm::Call { head, args } => {
            if head == "Application" {
                let mut ids: Vec<TermId> = args.iter().map(|a| form_to_term(session, a)).collect();
                if ids.is_empty() {
                    return push_matlab_call(session, head, ids);
                }
                let callee = ids.remove(0);
                let mut wrapped = vec![callee];
                wrapped.extend(ids);
                push_semantic(session, SemanticOperator::ApplyHead, wrapped)
            }
            else {
                let ids: Vec<TermId> = args.iter().map(|a| form_to_term(session, a)).collect();
                push_matlab_call(session, head, ids)
            }
        }
    }
}

/// Lift a [`MatlabForm`] into a neutral [`AthenaRequest`].
///
/// Request-shaped heads match on Form. Ordinary expressions become
/// [`AthenaRequest::Term`] via [`form_to_term`].
pub fn lower_request(session: &mut Session, form: &MatlabForm) -> AthenaRequest {
    match form {
        MatlabForm::Call { head, args } if head == "Set" => {
            if let [lhs, rhs] = args.as_slice() {
                if let Some(name) = form_symbol_name(lhs) {
                    let symbol = session.arena.symbols_mut().intern(name);
                    let value = form_to_term(session, rhs);
                    return AthenaRequest::Command(SessionCommand::Define {
                        symbol,
                        value,
                        kind: BindingKind::Session,
                        evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                    });
                }
            }
        }
        MatlabForm::Call { head, args } if head == "CompoundExpression" => {
            let steps: Vec<AthenaRequest> = args.iter().map(|a| lower_request(session, a)).collect();
            return AthenaRequest::Control(ControlPlan::Sequence { steps });
        }
        MatlabForm::Call { head, args } if head == "If" || head == "Branch" => match args.as_slice() {
            [cond, then_branch] => {
                return AthenaRequest::Control(ControlPlan::Branch {
                    condition: form_to_term(session, cond),
                    then_branch: Box::new(lower_request(session, then_branch)),
                    else_branch: None,
                });
            }
            [cond, then_branch, else_branch] => {
                return AthenaRequest::Control(ControlPlan::Branch {
                    condition: form_to_term(session, cond),
                    then_branch: Box::new(lower_request(session, then_branch)),
                    else_branch: Some(Box::new(lower_request(session, else_branch))),
                });
            }
            _ => {}
        },
        MatlabForm::Call { head, args } if head == "While" || head == "LoopWhile" => {
            if let [cond, body] = args.as_slice() {
                return AthenaRequest::Control(ControlPlan::LoopWhile {
                    condition: form_to_term(session, cond),
                    body: Box::new(lower_request(session, body)),
                });
            }
        }
        MatlabForm::Call { head, args } if head == "Try" || head == "Recover" => {
            if let [body, handler] = args.as_slice() {
                return AthenaRequest::Control(ControlPlan::Recover {
                    body: Box::new(lower_request(session, body)),
                    handler: Box::new(lower_request(session, handler)),
                });
            }
        }
        MatlabForm::Call { head, args } if head == "For" || head == "CountedLoop" => {
            if let [variable, iterator, body] = args.as_slice() {
                let variable_t = form_to_term(session, variable);
                let iterator_t = form_to_term(session, iterator);
                let body_req = lower_request(session, body);
                if matches!(body_req, AthenaRequest::Term(_)) {
                    return AthenaRequest::Control(ControlPlan::CountedLoop {
                        variable: variable_t,
                        iterator: iterator_t,
                        body: Box::new(body_req),
                    });
                }
                if let (Some(symbol), Some(items)) =
                    (symbol_atom(session, variable_t), expand_counted_iterator(session, iterator_t))
                {
                    let mut steps = Vec::with_capacity(items.len().saturating_mul(2));
                    for item in items {
                        steps.push(AthenaRequest::Command(SessionCommand::Define {
                            symbol,
                            value: item,
                            kind: BindingKind::Session,
                            evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                        }));
                        steps.push(lower_request(session, body));
                    }
                    return AthenaRequest::Control(ControlPlan::Sequence { steps });
                }
                return AthenaRequest::Control(ControlPlan::CountedLoop {
                    variable: variable_t,
                    iterator: iterator_t,
                    body: Box::new(body_req),
                });
            }
        }
        MatlabForm::Call { head, args } if head == "error" || head == "Error" || head == "Reject" => {
            let _ = args;
            return AthenaRequest::Control(ControlPlan::Reject);
        }
        MatlabForm::Call { head, args } if head == "Part" => {
            if args.len() >= 2 {
                let target = form_to_term(session, &args[0]);
                let axis_terms: Vec<TermId> = args[1..].iter().map(|a| form_to_term(session, a)).collect();
                if let Some(axes) = axis_terms.iter().copied().map(|a| index_spec_of(session, a)).collect::<Option<Vec<_>>>() {
                    return AthenaRequest::Control(ControlPlan::Index { target, axes });
                }
            }
        }
        MatlabForm::Call { head, args } if head == "Span" => {
            let rewritten = form_to_term(session, &MatlabForm::call("Range", args.clone()));
            return AthenaRequest::Term(rewritten);
        }
        MatlabForm::Call { head, args }
            if matches!(head.as_str(), "diff" | "Diff" | "D" | "int" | "Int" | "integral" | "Integrate") =>
        {
            match (head.as_str(), args.as_slice()) {
                ("diff" | "Diff" | "D", [expr, var]) => {
                    if let Some(name) = form_symbol_name(var) {
                        let variable = session.arena.symbols_mut().intern(name);
                        return calculus_goal(CalculusRequest::Derivative {
                            expression: form_to_term(session, expr),
                            variable,
                            order: DerivativeOrder::First,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("diff" | "Diff" | "D", [expr, var, order]) => {
                    if let Some(name) = form_symbol_name(var) {
                        let order_term = form_to_term(session, order);
                        if let Some(n) = number_from_id(session, order_term).and_then(|n| n.as_exact_integer()) {
                            if n > 0 {
                                let variable = session.arena.symbols_mut().intern(name);
                                let order = if n == 1 { DerivativeOrder::First } else { DerivativeOrder::Repeated(n as u32) };
                                return calculus_goal(CalculusRequest::Derivative {
                                    expression: form_to_term(session, expr),
                                    variable,
                                    order,
                                    assumptions: AssumptionSet::empty(),
                                });
                            }
                        }
                    }
                }
                ("int" | "Int" | "integral" | "Integrate", [expr, var]) => {
                    if let Some(name) = form_symbol_name(var) {
                        let variable = session.arena.symbols_mut().intern(name);
                        return calculus_goal(CalculusRequest::Integral {
                            expression: form_to_term(session, expr),
                            variable,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                _ => {}
            }
        }
        MatlabForm::Call { head, args } if head == "LinearSolve" || head == "Mldivide" => {
            if let [a_form, b_form] = args.as_slice() {
                let a_term = form_to_term(session, a_form);
                let b_term = form_to_term(session, b_form);
                if let (Some(a_mat), Some(b_mat)) =
                    (matrix_from_nested_list(session, a_term), matrix_from_nested_list(session, b_term))
                {
                    let a = session.matrix_objects.intern(a_mat);
                    let b = session.matrix_objects.intern(b_mat);
                    return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::Solve { a, b },
                    )));
                }
            }
        }
        _ => {}
    }
    // Ordinary expression Forms materialize once. Request-shaped heads above already returned.
    AthenaRequest::Term(form_to_term(session, form))
}

fn form_symbol_name(form: &MatlabForm) -> Option<&str> {
    match form {
        MatlabForm::Atom(MatlabAtom::Symbol(name)) => Some(name.as_str()),
        _ => None,
    }
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
