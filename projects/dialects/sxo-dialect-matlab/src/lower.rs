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
        linear_algebra::{MatrixOperand, MatrixValue},
    },
    ir::{Atom, MathematicalConstant, SemanticOperator, TermNode},
    numeric::{Integer, Rational},
    runtime::{
        ZeroPowerZeroConvention,
        values::{
            arena::{
                application_arguments, number_from_id, push_bool, push_constant, push_list, push_null, push_semantic,
                push_symbol_name, symbol_name,
            },
            numeric_clone::{clone_integer, clone_number, clone_rational},
        },
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

/// Install MATLAB exact-domain numeric conventions on an Athena session.
///
/// Selects [`ZeroPowerZeroConvention::One`] so evaluated zeros (`x=0; x^0`) match
/// literal `0^0`, without Form-level literal special-cases.
pub fn install_session_conventions(session: &mut Session) {
    session.zero_pow_zero = ZeroPowerZeroConvention::One;
}

/// Run `f` with MATLAB conventions installed, then restore the prior `0^0` policy.
pub fn with_session_conventions<R>(session: &mut Session, f: impl FnOnce(&mut Session) -> R) -> R {
    let previous = session.zero_pow_zero;
    install_session_conventions(session);
    let out = f(session);
    session.zero_pow_zero = previous;
    out
}

/// Materialize a [`MatlabForm`] into the session arena (transitional bridge).
pub fn form_to_term(session: &mut Session, form: &MatlabForm) -> TermId {
    match form {
        MatlabForm::Atom(MatlabAtom::Number(n)) => {
            session.arena.push(TermNode::Atom(Atom::Number(clone_number(n))), SourceSpan::default())
        }
        MatlabForm::Atom(MatlabAtom::String(s)) => {
            session.arena.push(TermNode::Atom(Atom::String(s.clone())), SourceSpan::default())
        }
        MatlabForm::Atom(MatlabAtom::Symbol(name)) => match name.as_str() {
            // Dialect surface `Inf` → typed positive infinity constant.
            "Inf" | "inf" => push_constant(session, MathematicalConstant::Infinity),
            // Dialect surface `NaN` → neutral `Indeterminate`.
            "NaN" | "nan" => push_semantic(session, SemanticOperator::Indeterminate, Vec::new()),
            _ => push_symbol_name(session, name),
        },
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
            else if head == "feval" {
                // feval(@sin, 0) → Sin(0) via mapped FunctionHandle target.
                if let Some(MatlabForm::Call { head: h, args: handle_args }) = args.first() {
                    if h == "FunctionHandle" {
                        if let Some(target) = handle_args.first() {
                            if let Some(name) = form_symbol_name(target) {
                                let rest: Vec<MatlabForm> = args.iter().skip(1).cloned().collect();
                                return form_to_term(session, &MatlabForm::call(name, rest));
                            }
                            let mut wrapped = vec![target.clone()];
                            wrapped.extend(args.iter().skip(1).cloned());
                            return form_to_term(session, &MatlabForm::call("Application", wrapped));
                        }
                    }
                }
                let ids: Vec<TermId> = args.iter().map(|a| form_to_term(session, a)).collect();
                push_matlab_call(session, head, ids)
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
                    // List / integer Range Form → matrix Own（含行/列向量与 `1:n`）。
                    if let Some(mat) = matrix_from_form(rhs).or_else(|| matrix_from_range_form(rhs)) {
                        let matrix = session.matrix_objects.intern(mat);
                        return AthenaRequest::Command(SessionCommand::DefineMatrix { symbol, matrix });
                    }
                    let value = form_to_term(session, rhs);
                    return AthenaRequest::Command(SessionCommand::Define {
                        symbol,
                        value,
                        kind: BindingKind::Session,
                        evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                    });
                }
                // `A(2)=9` → StoreIndex (typed write). Refuse residual Set that silently no-ops Own.
                if let MatlabForm::Call { head: part_head, args: part_args } = lhs {
                    if part_head == "Part" && part_args.len() >= 2 {
                        if let Some(name) = form_symbol_name(&part_args[0]) {
                            let target = push_symbol_name(session, name);
                            let axis_terms: Vec<TermId> =
                                part_args[1..].iter().map(|a| form_to_term(session, a)).collect();
                            if let Some(axes) = axis_terms
                                .iter()
                                .copied()
                                .enumerate()
                                .map(|(i, a)| index_spec_of(session, a, axis_terms.len() == 1 && i == 0))
                                .collect::<Option<Vec<_>>>()
                            {
                                let value = form_to_term(session, rhs);
                                return AthenaRequest::Control(ControlPlan::StoreIndex { target, axes, value });
                            }
                        }
                    }
                }
                return AthenaRequest::Control(ControlPlan::Reject);
            }
        }
        MatlabForm::Call { head, args } if head == "Times" => {
            // Living 16: MATLAB `*` is MatMul. Element-wise is `.*` / DotTimes.
            if let [a_form, b_form] = args.as_slice() {
                if let (Some(lhs), Some(rhs)) = (matrix_operand_from_form(session, a_form), matrix_operand_from_form(session, b_form)) {
                    return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::MatMul { lhs, rhs },
                    )));
                }
            }
        }
        MatlabForm::Call { head, args } if head == "DotTimes" => {
            // Living 16: MATLAB `.*` is Hadamard on typed matrices.
            if let [a_form, b_form] = args.as_slice() {
                if let (Some(lhs), Some(rhs)) = (matrix_operand_from_form(session, a_form), matrix_operand_from_form(session, b_form)) {
                    return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::Hadamard { lhs, rhs },
                    )));
                }
            }
        }
        MatlabForm::Call { head, args } if head == "DotDivide" => {
            // Living 16: MATLAB `./` is ElementwiseDivide on typed matrices.
            if let [a_form, b_form] = args.as_slice() {
                if let (Some(lhs), Some(rhs)) = (matrix_operand_from_form(session, a_form), matrix_operand_from_form(session, b_form)) {
                    return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::ElementwiseDivide { lhs, rhs },
                    )));
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
        MatlabForm::Call { head, args } if head == "And" => {
            return lower_short_circuit_and(session, args);
        }
        MatlabForm::Call { head, args } if head == "Or" => {
            return lower_short_circuit_or(session, args);
        }
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
        MatlabForm::Call { head, args } if head == "Global" || head == "Persistent" || head == "Command" || head == "Member" || head == "Parfor" || head == "Spmd" => {
            // Typed declarations / command / member / parallel Forms — no silent strip.
            let _ = args;
            return AthenaRequest::Control(ControlPlan::Reject);
        }
        MatlabForm::Call { head, args } if head == "Application" => {
            // Package / member calls `containers.Map(...)` — refuse until runtime exists.
            if args.first().is_some_and(|a| a.head_name() == Some("Member")) {
                return AthenaRequest::Control(ControlPlan::Reject);
            }
        }
        MatlabForm::Call { head, args } if head == "Part" => {
            if args.len() >= 2 {
                let target = form_to_term(session, &args[0]);
                let axis_terms: Vec<TermId> = args[1..].iter().map(|a| form_to_term(session, a)).collect();
                if let Some(axes) = axis_terms
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(i, a)| index_spec_of(session, a, axis_terms.len() == 1 && i == 0))
                    .collect::<Option<Vec<_>>>()
                {
                    return AthenaRequest::Control(ControlPlan::Index { target, axes });
                }
            }
        }
        MatlabForm::Call { head, args } if head == "Transpose" || head == "ConjugateTranspose" => {
            // Real matrices: conjugate transpose equals transpose. Complex ctranspose is later.
            if let [arg] = args.as_slice() {
                // True 2-D MatrixValue path from Form lists; row/column vectors keep Term reshape.
                if let Some(mat) = matrix_from_form(arg) {
                    let shape = mat.shape();
                    if shape.rows > 1 && shape.cols > 1 {
                        let matrix = session.matrix_objects.intern(mat);
                        return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                            athena::domains::linear_algebra::LinearAlgebraRequest::Transpose { matrix: matrix.into() },
                        )));
                    }
                }
                if let Some(name) = form_symbol_name(arg) {
                    let symbol = session.arena.symbols_mut().intern(name);
                    return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::Transpose {
                            matrix: MatrixOperand::binding(symbol),
                        },
                    )));
                }
                let term = form_to_term(session, arg);
                if let Some(transposed) = transpose_nested_or_vector(session, term) {
                    return AthenaRequest::Term(transposed);
                }
            }
        }
        MatlabForm::Call { head, args } if head == "Det" || head == "Determinant" => {
            if let [arg] = args.as_slice() {
                if let Some(matrix) = matrix_operand_from_form(session, arg) {
                    return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                        athena::domains::linear_algebra::LinearAlgebraRequest::Det { matrix },
                    )));
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
                if let (Some(a), Some(b)) = (matrix_operand_from_form(session, a_form), matrix_operand_from_form(session, b_form)) {
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

/// Short-circuit `&&` (`And`): later arms run only when earlier conditions are true.
///
/// Scalar MATLAB `&&` yields a logical. Pure last arms are coerced via nested
/// `Branch`. Assignment / compound last arms keep `lower_request` so side effects run.
fn lower_short_circuit_and(session: &mut Session, args: &[MatlabForm]) -> AthenaRequest {
    match args {
        [] => AthenaRequest::Term(push_bool(session, true)),
        [only] if form_is_effectful_arm(only) => lower_request(session, only),
        [only] => AthenaRequest::Control(ControlPlan::Branch {
            condition: form_to_term(session, only),
            then_branch: Box::new(AthenaRequest::Term(push_bool(session, true))),
            else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, false)))),
        }),
        [first, rest @ ..] => AthenaRequest::Control(ControlPlan::Branch {
            condition: form_to_term(session, first),
            then_branch: Box::new(lower_short_circuit_and(session, rest)),
            else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, false)))),
        }),
    }
}

/// Short-circuit `||` (`Or`): later arms run only when earlier conditions are false.
fn lower_short_circuit_or(session: &mut Session, args: &[MatlabForm]) -> AthenaRequest {
    match args {
        [] => AthenaRequest::Term(push_bool(session, false)),
        [only] if form_is_effectful_arm(only) => lower_request(session, only),
        [only] => AthenaRequest::Control(ControlPlan::Branch {
            condition: form_to_term(session, only),
            then_branch: Box::new(AthenaRequest::Term(push_bool(session, true))),
            else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, false)))),
        }),
        [first, rest @ ..] => AthenaRequest::Control(ControlPlan::Branch {
            condition: form_to_term(session, first),
            then_branch: Box::new(AthenaRequest::Term(push_bool(session, true))),
            else_branch: Some(Box::new(lower_short_circuit_or(session, rest))),
        }),
    }
}

fn form_is_effectful_arm(form: &MatlabForm) -> bool {
    match form {
        MatlabForm::Call { head, .. } if head == "Set" || head == "CompoundExpression" => true,
        _ => false,
    }
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

fn form_scalar_rational(w: &MatlabForm) -> Option<Rational> {
    match w {
        MatlabForm::Atom(MatlabAtom::Number(n)) => {
            if let Some(i) = n.as_exact_integer() {
                return Some(Rational::new(Integer::from_i64(i), Integer::one()));
            }
            if let Some(i) = n.as_integer() {
                return Some(Rational::from_integer(clone_integer(i)));
            }
            n.as_rational().map(clone_rational)
        }
        _ => None,
    }
}

fn form_list_items(w: &MatlabForm) -> Option<&[MatlabForm]> {
    match w {
        MatlabForm::List(items) => Some(items.as_slice()),
        _ => None,
    }
}

/// Literal Form matrix or symbol Own binding for linear-algebra goals.
fn matrix_operand_from_form(session: &mut Session, w: &MatlabForm) -> Option<MatrixOperand> {
    if let Some(mat) = matrix_from_form(w) {
        Some(MatrixOperand::object(session.matrix_objects.intern(mat)))
    }
    else if let Some(name) = form_symbol_name(w) {
        Some(MatrixOperand::binding(session.arena.symbols_mut().intern(name)))
    }
    else {
        None
    }
}

/// Build a dense rational `MatrixValue` from MATLAB list Form literals.
///
/// Nested row lists → 2-D matrix. Flat list → `1×n` row. Does not reverse-recognize
/// arena Collections from variables or computed terms.
fn matrix_from_form(w: &MatlabForm) -> Option<MatrixValue> {
    let rows = form_list_items(w)?;
    if rows.is_empty() {
        return None;
    }
    if form_list_items(&rows[0]).is_some() {
        let mut data = Vec::new();
        let mut cols: Option<u64> = None;
        for row in rows {
            let cells = form_list_items(row)?;
            let c = cells.len() as u64;
            match cols {
                Some(prev) if prev != c => return None,
                None => cols = Some(c),
                _ => {}
            }
            for cell in cells {
                data.push(form_scalar_rational(cell)?);
            }
        }
        MatrixValue::from_rationals_row_major(rows.len() as u64, cols.unwrap_or(0), data).ok()
    } else {
        let mut data = Vec::with_capacity(rows.len());
        for cell in rows {
            data.push(form_scalar_rational(cell)?);
        }
        MatrixValue::from_rationals_row_major(1, data.len() as u64, data).ok()
    }
}

/// Integer `Range` / `start:end` / `start:step:end` Form → `1×n` matrix Own.
///
/// Builds from Form scalars only（禁止 Term Collection 反向识别）。
fn matrix_from_range_form(w: &MatlabForm) -> Option<MatrixValue> {
    let MatlabForm::Call { head, args } = w
    else {
        return None;
    };
    if head != "Range" {
        return None;
    }
    let values = match args.as_slice() {
        [start, end] => {
            let a = form_scalar_i64(start)?;
            let b = form_scalar_i64(end)?;
            expand_span(a, 1, b)?
        }
        [start, end, step] => {
            // Parse already rewrites MATLAB `start:step:end` → `Range[start, end, step]`.
            let a = form_scalar_i64(start)?;
            let b = form_scalar_i64(end)?;
            let s = form_scalar_i64(step)?;
            expand_span(a, s, b)?
        }
        _ => return None,
    };
    if values.is_empty() {
        return None;
    }
    let data: Vec<Rational> = values.into_iter().map(|i| Rational::new(Integer::from_i64(i), Integer::one())).collect();
    MatrixValue::from_rationals_row_major(1, data.len() as u64, data).ok()
}

fn form_scalar_i64(w: &MatlabForm) -> Option<i64> {
    match w {
        MatlabForm::Atom(MatlabAtom::Number(n)) => n.as_exact_integer(),
        _ => None,
    }
}

fn index_spec_of(session: &Session, term: TermId, single_axis: bool) -> Option<IndexSpec> {
    if let Some(n) = number_from_id(session, term).and_then(|n| n.as_exact_integer()) {
        // MATLAB `A(k)` is column-major linear on matrices; Athena rewrites at Index time.
        if single_axis {
            return Some(IndexSpec::LinearColumnMajor(IntegerIndex(n)));
        }
        return Some(IndexSpec::Scalar(IntegerIndex(n)));
    }
    match symbol_name(session, term).as_deref() {
        Some("All") | Some(":") => {
            // MATLAB `A(:)` flattens column-major; `A(1,:)` keeps axis `All`.
            if single_axis {
                return Some(IndexSpec::ColumnMajorFlatten);
            }
            return Some(IndexSpec::All);
        }
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

/// Transpose nested-list matrices and flat row/column vectors (MATLAB `.'` / real `'`).
fn transpose_nested_or_vector(session: &mut Session, term: TermId) -> Option<TermId> {
    let rows = match session.arena.get(term)? {
        TermNode::Collection { elements, .. } => elements.clone(),
        _ => return None,
    };
    if rows.is_empty() {
        return Some(term);
    }
    if matches!(session.arena.get(rows[0]), Some(TermNode::Collection { .. })) {
        let row_vecs: Option<Vec<Vec<TermId>>> = rows
            .iter()
            .map(|r| match session.arena.get(*r) {
                Some(TermNode::Collection { elements, .. }) => Some(elements.clone()),
                _ => None,
            })
            .collect();
        let row_vecs = row_vecs?;
        let nrows = row_vecs.len();
        let ncols = row_vecs.first()?.len();
        if row_vecs.iter().any(|r| r.len() != ncols) {
            return None;
        }
        if ncols == 1 {
            // Column vector → flat row.
            let flat: Vec<TermId> = row_vecs.into_iter().map(|r| r[0]).collect();
            return Some(push_list(session, flat));
        }
        let mut out_rows = Vec::with_capacity(ncols);
        for c in 0..ncols {
            let mut row = Vec::with_capacity(nrows);
            for r in 0..nrows {
                row.push(row_vecs[r][c]);
            }
            out_rows.push(push_list(session, row));
        }
        Some(push_list(session, out_rows))
    }
    else {
        // Flat row → column of singleton rows.
        let mut out_rows = Vec::with_capacity(rows.len());
        for e in rows {
            out_rows.push(push_list(session, vec![e]));
        }
        Some(push_list(session, out_rows))
    }
}
