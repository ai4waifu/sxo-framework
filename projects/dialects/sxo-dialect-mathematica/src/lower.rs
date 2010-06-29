//! Lower Mathematica Form ([`WolframForm`]) into Athena session terms / requests (Living `14`).

use athena::{
    Session,
    api::{AthenaRequest, ControlPlan, DomainGoal, SessionCommand},
    domains::{
        DomainRequest,
        calculus::{CalculusRequest, DerivativeOrder, LimitApproach, LimitDirection, TransformKind},
        linear_algebra::MatrixValue,
    },
    ir::{ApplicationHead, Atom, MathematicalConstant, SemanticOperator, TermNode, UnaryFunction},
    numeric::{Integer, Rational},
    reasoning::trs::{PatternConstraint, TermPattern},
    runtime::values::{
        arena::{
            number_from_id, push_bool, push_constant, push_extension, push_int, push_list, push_null, push_semantic,
            push_symbol_name,
        },
        numeric_clone::{clone_integer, clone_number, clone_rational},
    },
    types::{
        AssumptionSet, BindingEvaluationPolicy, BindingKind, IndexSpec, IntegerIndex, IntegerOffset, SymbolId, TermId,
        ValueTypeId,
    },
};

use crate::form::{WolframAtom, WolframForm};

/// Map a Mathematica surface head to a closed [`SemanticOperator`] when known.
///
/// Only closed Athena operators belong here (Living `07`). Do **not** rewrite list
/// structure into concrete Form trees in this crate to fake evaluation — that
/// belongs in Athena kernels. Surface aliases such as `Total` → `Sum` are OK.
pub fn surface_to_semantic(name: &str) -> Option<SemanticOperator> {
    Some(match name {
        "Plus" => SemanticOperator::Add,
        "Times" => SemanticOperator::Multiply,
        "Subtract" => SemanticOperator::Subtract,
        "Divide" => SemanticOperator::Divide,
        "Power" => SemanticOperator::Power,
        "Minus" => SemanticOperator::Negate,
        "Equal" => SemanticOperator::Equal,
        "Unequal" => SemanticOperator::Unequal,
        "Less" => SemanticOperator::Less,
        "Greater" => SemanticOperator::Greater,
        "LessEqual" => SemanticOperator::LessEqual,
        "GreaterEqual" => SemanticOperator::GreaterEqual,
        "And" => SemanticOperator::And,
        "Or" => SemanticOperator::Or,
        "Not" => SemanticOperator::Not,
        "Range" | "Span" => SemanticOperator::Range,
        "Apply" => SemanticOperator::Apply,
        "Map" => SemanticOperator::Map,
        "Rule" => SemanticOperator::Rule,
        "RuleDelayed" => SemanticOperator::RuleDeferred,
        "ReplaceAll" => SemanticOperator::ReplaceAll,
        "Simplify" => SemanticOperator::Simplify,
        "Hold" | "HoldForm" => SemanticOperator::Hold,
        "Function" => SemanticOperator::Function,
        "Factorial" => SemanticOperator::Factorial,
        "Length" => SemanticOperator::Length,
        "First" => SemanticOperator::First,
        "Rest" => SemanticOperator::Rest,
        "Join" => SemanticOperator::Join,
        "Sum" => SemanticOperator::Sum,
        "Total" => SemanticOperator::Sum,
        "Product" => SemanticOperator::Product,
        "Determinant" | "Det" => SemanticOperator::Determinant,
        "DotTimes" => SemanticOperator::ElementwiseMultiply,
        "DotDivide" => SemanticOperator::ElementwiseDivide,
        "DotPower" => SemanticOperator::ElementwisePower,
        "Sin" => SemanticOperator::from_unary(UnaryFunction::Sin),
        "Cos" => SemanticOperator::from_unary(UnaryFunction::Cos),
        "Tan" => SemanticOperator::from_unary(UnaryFunction::Tan),
        "Exp" => SemanticOperator::from_unary(UnaryFunction::Exp),
        "Log" => SemanticOperator::from_unary(UnaryFunction::Log),
        "Sinh" => SemanticOperator::from_unary(UnaryFunction::Sinh),
        "Cosh" => SemanticOperator::from_unary(UnaryFunction::Cosh),
        "Tanh" => SemanticOperator::from_unary(UnaryFunction::Tanh),
        "ArcSin" => SemanticOperator::from_unary(UnaryFunction::ArcSin),
        "ArcCos" => SemanticOperator::from_unary(UnaryFunction::ArcCos),
        "ArcTan" => SemanticOperator::from_unary(UnaryFunction::ArcTan),
        "Sqrt" => SemanticOperator::from_unary(UnaryFunction::Sqrt),
        "Abs" => SemanticOperator::from_unary(UnaryFunction::Abs),
        "Sign" => SemanticOperator::from_unary(UnaryFunction::Sign),
        "Gamma" => SemanticOperator::from_unary(UnaryFunction::Gamma),
        "Erf" => SemanticOperator::from_unary(UnaryFunction::Erf),
        _ => return None,
    })
}

/// Map a closed semantic op back to a Mathematica surface head for Form/render.
pub fn semantic_to_surface(op: SemanticOperator) -> &'static str {
    match op {
        SemanticOperator::Add => "Plus",
        SemanticOperator::Multiply => "Times",
        SemanticOperator::Negate => "Minus",
        SemanticOperator::RuleDeferred => "RuleDelayed",
        SemanticOperator::ElementwiseMultiply => "DotTimes",
        SemanticOperator::ElementwiseDivide => "DotDivide",
        SemanticOperator::ElementwisePower => "DotPower",
        SemanticOperator::ApplyHead => "Application",
        SemanticOperator::Unary(f) => f.debug_label(),
        other => other.debug_label(),
    }
}

/// Push a Mathematica surface call as Semantic when mapped, else Extension.
pub fn push_surface_call(session: &mut Session, name: &str, args: Vec<TermId>) -> TermId {
    if let Some(op) = surface_to_semantic(name) {
        push_semantic(session, op, args)
    }
    else {
        let op = session.extensions.intern(name);
        push_extension(session, op, args)
    }
}

/// Structural `WolframForm` → session arena [`TermId`].
///
/// Prefer [`lower_request`] when the form carries session / control semantics.
pub fn lower_wexpr(session: &mut Session, w: &WolframForm) -> TermId {
    match w {
        WolframForm::Atom(a) => match a {
            WolframAtom::Number(n) => {
                let span = athena::types::SourceSpan::default();
                session.arena.push(TermNode::Atom(Atom::Number(clone_number(n))), span)
            }
            WolframAtom::String(s) => {
                let span = athena::types::SourceSpan::default();
                session.arena.push(TermNode::Atom(Atom::String(s.clone())), span)
            }
            WolframAtom::Symbol(s) if s == "True" => push_bool(session, true),
            WolframAtom::Symbol(s) if s == "False" => push_bool(session, false),
            WolframAtom::Symbol(s) if s == "Null" => push_null(session),
            WolframAtom::Symbol(s) if s == "Pi" => push_constant(session, MathematicalConstant::Pi),
            WolframAtom::Symbol(s) if s == "E" => push_constant(session, MathematicalConstant::EulerNumber),
            WolframAtom::Symbol(s) => push_symbol_name(session, s),
        },
        WolframForm::List(items) => {
            let ids: Vec<TermId> = items.iter().map(|i| lower_wexpr(session, i)).collect();
            push_list(session, ids)
        }
        WolframForm::Call { head, args } => match head.as_ref() {
            WolframForm::Atom(WolframAtom::Symbol(name)) if name == "Function" => lower_function(session, args),
            WolframForm::Atom(WolframAtom::Symbol(name)) if name == "Span" => lower_span_as_range(session, args),
            WolframForm::Atom(WolframAtom::Symbol(name)) if name == "Apply" || name == "Map" => {
                let mut arg_ids = Vec::with_capacity(args.len());
                for (i, a) in args.iter().enumerate() {
                    if i == 0 {
                        arg_ids.push(lower_operator_value(session, a));
                    }
                    else {
                        arg_ids.push(lower_wexpr(session, a));
                    }
                }
                push_surface_call(session, name, arg_ids)
            }
            WolframForm::Atom(WolframAtom::Symbol(name)) => {
                let arg_ids: Vec<TermId> = args.iter().map(|a| lower_wexpr(session, a)).collect();
                push_surface_call(session, name, arg_ids)
            }
            other => {
                let h = lower_wexpr(session, other);
                let mut wrapped = vec![h];
                wrapped.extend(args.iter().map(|a| lower_wexpr(session, a)));
                push_semantic(session, SemanticOperator::ApplyHead, wrapped)
            }
        },
    }
}

/// Lower a head used as an operator value (`Apply[Plus, …]` → 0-ary `Add`).
fn lower_operator_value(session: &mut Session, w: &WolframForm) -> TermId {
    match w {
        WolframForm::Atom(WolframAtom::Symbol(name)) => {
            if let Some(op) = surface_to_semantic(name) {
                push_semantic(session, op, Vec::new())
            }
            else {
                push_symbol_name(session, name)
            }
        }
        other => lower_wexpr(session, other),
    }
}

/// Dialect Form → neutral [`AthenaRequest`].
///
/// Maps Mathematica surface assignment / iteration into Session / Control contracts,
/// and calculus surface (`D` / `Integrate` / `Limit`) into [`AthenaRequest::Goal`].
/// Other forms lower to [`AthenaRequest::Term`].
pub fn lower_request(session: &mut Session, w: &WolframForm) -> AthenaRequest {
    if let WolframForm::Call { head, args } = w {
        if let WolframForm::Atom(WolframAtom::Symbol(name)) = head.as_ref() {
            match (name.as_str(), args.as_slice()) {
                ("D", [expr, spec]) => {
                    if let Some(variable) = symbol_of(session, spec) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Derivative {
                            expression,
                            variable,
                            order: DerivativeOrder::First,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                    if let Some((variable, order)) = derivative_spec(session, spec) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Derivative {
                            expression,
                            variable,
                            order,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Integrate", [expr, spec]) => {
                    if let Some(variable) = symbol_of(session, spec) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Integral {
                            expression,
                            variable,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                    if let Some((variable, lower, upper)) = definite_integral_spec(session, spec) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::DefiniteIntegral {
                            expression,
                            variable,
                            lower,
                            upper,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Limit", [expr, rule]) => {
                    if let Some((variable, approach, direction)) = limit_rule(session, rule) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Limit {
                            expression,
                            variable,
                            approach,
                            direction,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Series", [expr, spec]) => {
                    if let Some((variable, center, order)) = series_spec(session, spec) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Series {
                            expression,
                            variable,
                            center,
                            order,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Residue", [expr, spec]) => {
                    if let Some((variable, point)) = residue_spec(session, spec) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Residue {
                            expression,
                            variable,
                            point,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Grad", [expr, vars]) => {
                    if let Some(variables) = symbol_list(session, vars) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Gradient {
                            expression,
                            variables,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Div", [field, vars]) => {
                    if let (Some(components), Some(variables)) = (term_list(session, field), symbol_list(session, vars)) {
                        return calculus_goal(CalculusRequest::Divergence {
                            components,
                            variables,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Curl", [field, vars]) => {
                    if let (Some(components), Some(variables)) = (term_list(session, field), symbol_list(session, vars)) {
                        return calculus_goal(CalculusRequest::Curl {
                            components,
                            variables,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("LaplaceTransform", [expr, time, transform]) => {
                    if let (Some(time_variable), Some(transform_variable)) = (symbol_of(session, time), symbol_of(session, transform)) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Transform {
                            kind: TransformKind::Laplace,
                            expression,
                            time_variable,
                            transform_variable,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("FourierTransform", [expr, time, transform]) => {
                    if let (Some(time_variable), Some(transform_variable)) = (symbol_of(session, time), symbol_of(session, transform)) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Transform {
                            kind: TransformKind::Fourier,
                            expression,
                            time_variable,
                            transform_variable,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("ZTransform", [expr, time, transform]) => {
                    if let (Some(time_variable), Some(transform_variable)) = (symbol_of(session, time), symbol_of(session, transform)) {
                        let expression = lower_wexpr(session, expr);
                        return calculus_goal(CalculusRequest::Transform {
                            kind: TransformKind::Z,
                            expression,
                            time_variable,
                            transform_variable,
                            assumptions: AssumptionSet::empty(),
                        });
                    }
                }
                ("Solve", [equation, unknown]) => {
                    if let Some(unknown) = symbol_of(session, unknown) {
                        let equation = lower_wexpr(session, equation);
                        return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::Solve(
                            athena::domains::solve::SolveRequest::UnivariateEquation { equation, unknown },
                        )));
                    }
                }
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
                ("Clear", args) => {
                    let mut steps = Vec::with_capacity(args.len());
                    let mut ok = true;
                    for a in args {
                        match symbol_of(session, a) {
                            Some(symbol) => steps.push(AthenaRequest::Command(SessionCommand::ClearDefinition { symbol })),
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok && !steps.is_empty() {
                        return if steps.len() == 1 {
                            steps.remove(0)
                        }
                        else {
                            AthenaRequest::Control(ControlPlan::Sequence { steps })
                        };
                    }
                }
                ("SetDelayed", [lhs, rhs]) => {
                    if matches!(lhs, WolframForm::Atom(WolframAtom::Symbol(_))) {
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
                    // Patterned down-value: `f[x_]:=…` → typed `TermPattern` dispatch (Living `14`).
                    if let WolframForm::Call { head, args } = lhs {
                        if let WolframForm::Atom(WolframAtom::Symbol(name)) = head.as_ref() {
                            let mut pat_args = Vec::with_capacity(args.len());
                            let mut ok = true;
                            for a in args {
                                match wexpr_to_term_pattern(session, a) {
                                    Some(p) => pat_args.push(p),
                                    None => {
                                        ok = false;
                                        break;
                                    }
                                }
                            }
                            if ok {
                                let f_op = session.extensions.intern(name);
                                let pattern = TermPattern::Application {
                                    operator: ApplicationHead::Extension(f_op),
                                    arguments: pat_args,
                                };
                                let value = lower_wexpr(session, rhs);
                                session.defs.register_extension_rule(f_op, pattern, value);
                                return AthenaRequest::Term(push_null(session));
                            }
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
                ("CompoundExpression", args) => {
                    let steps: Vec<AthenaRequest> = args.iter().map(|a| lower_request(session, a)).collect();
                    return AthenaRequest::Control(ControlPlan::Sequence { steps });
                }
                ("If", [cond, then_branch]) => {
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, cond),
                        then_branch: Box::new(lower_request(session, then_branch)),
                        else_branch: None,
                    });
                }
                ("If", [cond, then_branch, else_branch]) => {
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, cond),
                        then_branch: Box::new(lower_request(session, then_branch)),
                        else_branch: Some(Box::new(lower_request(session, else_branch))),
                    });
                }
                ("Which", args) => {
                    let mut arms = Vec::new();
                    let mut otherwise = None;
                    let mut i = 0;
                    while i + 1 < args.len() {
                        let cond = lower_wexpr(session, &args[i]);
                        let branch = Box::new(lower_request(session, &args[i + 1]));
                        arms.push((cond, branch));
                        i += 2;
                    }
                    if i < args.len() {
                        otherwise = Some(Box::new(lower_request(session, &args[i])));
                    }
                    return AthenaRequest::Control(ControlPlan::Cond { arms, otherwise });
                }
                ("While", [cond, body]) => {
                    return AthenaRequest::Control(ControlPlan::LoopWhile {
                        condition: lower_wexpr(session, cond),
                        body: Box::new(lower_request(session, body)),
                    });
                }
                ("ReleaseHold" | "Evaluate", [inner]) => {
                    if let Some(held) = unwrap_hold_form(inner) {
                        return lower_request(session, held);
                    }
                }
                ("Assert", [cond]) => {
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, cond),
                        then_branch: Box::new(AthenaRequest::Term(push_null(session))),
                        else_branch: Some(Box::new(AthenaRequest::Control(ControlPlan::Reject))),
                    });
                }
                ("TrueQ", [cond]) => {
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, cond),
                        then_branch: Box::new(AthenaRequest::Term(push_bool(session, true))),
                        else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, false)))),
                    });
                }
                ("Boole", [cond]) => {
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, cond),
                        then_branch: Box::new(AthenaRequest::Term(push_int(session, 1))),
                        else_branch: Some(Box::new(AthenaRequest::Term(push_int(session, 0)))),
                    });
                }
                ("Implies", [antecedent, consequent]) => {
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, antecedent),
                        then_branch: Box::new(lower_request(session, consequent)),
                        else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, true)))),
                    });
                }
                ("And", args) => {
                    return lower_short_circuit_and(session, args);
                }
                ("Or", args) => {
                    return lower_short_circuit_or(session, args);
                }
                ("Xor", [left, right]) => {
                    let right_term = lower_wexpr(session, right);
                    return AthenaRequest::Control(ControlPlan::Branch {
                        condition: lower_wexpr(session, left),
                        then_branch: Box::new(AthenaRequest::Control(ControlPlan::Branch {
                            condition: right_term,
                            then_branch: Box::new(AthenaRequest::Term(push_bool(session, false))),
                            else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, true)))),
                        })),
                        else_branch: Some(Box::new(AthenaRequest::Control(ControlPlan::Branch {
                            condition: right_term,
                            then_branch: Box::new(AthenaRequest::Term(push_bool(session, true))),
                            else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, false)))),
                        }))),
                    });
                }
                ("Do", [body, iter]) => {
                    if let Some((variable, iterator)) = do_loop_parts(session, iter) {
                        let counted = AthenaRequest::Control(ControlPlan::CountedLoop {
                            variable,
                            iterator,
                            body: Box::new(lower_request(session, body)),
                        });
                        // Mathematica `Do` evaluates to `Null`.
                        return AthenaRequest::Control(ControlPlan::Sequence {
                            steps: vec![counted, AthenaRequest::Term(push_null(session))],
                        });
                    }
                }
                ("Module", [bindings, body]) => {
                    return lower_module(session, bindings, body);
                }
                ("Block", [bindings, body]) => {
                    return lower_block(session, bindings, body);
                }
                ("With", [bindings, body]) => {
                    return lower_with(session, bindings, body);
                }
                ("Part", args) if args.len() >= 2 => {
                    if let Some(axes) = args[1..].iter().map(index_spec_of).collect::<Option<Vec<_>>>() {
                        return AthenaRequest::Control(ControlPlan::Index { target: lower_wexpr(session, &args[0]), axes });
                    }
                }
                ("Transpose" | "ConjugateTranspose", [arg]) => {
                    // Real matrices: ConjugateTranspose equals Transpose. Complex path later.
                    let term = lower_wexpr(session, arg);
                    if let Some(mat) = matrix_from_nested_list(session, term) {
                        let shape = mat.shape();
                        if shape.rows >= 1 && shape.cols >= 1 {
                            let matrix = session.matrix_objects.intern(mat);
                            return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                                athena::domains::linear_algebra::LinearAlgebraRequest::Transpose { matrix },
                            )));
                        }
                    }
                }
                ("LinearSolve", [a_form, b_form]) => {
                    let a_term = lower_wexpr(session, a_form);
                    let b_term = lower_wexpr(session, b_form);
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
                ("MatrixRank", [arg]) => {
                    let term = lower_wexpr(session, arg);
                    if let Some(mat) = matrix_from_nested_list(session, term) {
                        let matrix = session.matrix_objects.intern(mat);
                        return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                            athena::domains::linear_algebra::LinearAlgebraRequest::Rank { matrix },
                        )));
                    }
                }
                ("RowReduce", [arg]) => {
                    let term = lower_wexpr(session, arg);
                    if let Some(mat) = matrix_from_nested_list(session, term) {
                        let matrix = session.matrix_objects.intern(mat);
                        return AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::LinearAlgebra(
                            athena::domains::linear_algebra::LinearAlgebraRequest::Rref { matrix },
                        )));
                    }
                }
                ("MatchQ", [expr, pat]) => {
                    if let Some(pattern) = wexpr_to_term_pattern(session, pat) {
                        return AthenaRequest::Control(ControlPlan::Match { target: lower_wexpr(session, expr), pattern });
                    }
                }
                ("Cases", [source, pat]) => {
                    if let Some(pattern) = wexpr_to_term_pattern(session, pat) {
                        return AthenaRequest::Control(ControlPlan::CollectMatches {
                            source: lower_wexpr(session, source),
                            pattern,
                        });
                    }
                }
                _ => {}
            }
        }
    }
    AthenaRequest::Term(lower_wexpr(session, w))
}

/// `And` as CFG short-circuit: later arms run only when earlier conditions are true.
fn lower_short_circuit_and(session: &mut Session, args: &[WolframForm]) -> AthenaRequest {
    match args {
        [] => AthenaRequest::Term(push_bool(session, true)),
        [only] => lower_request(session, only),
        [first, rest @ ..] => AthenaRequest::Control(ControlPlan::Branch {
            condition: lower_wexpr(session, first),
            then_branch: Box::new(lower_short_circuit_and(session, rest)),
            else_branch: Some(Box::new(AthenaRequest::Term(push_bool(session, false)))),
        }),
    }
}

/// `Or` as CFG short-circuit: later arms run only when earlier conditions are false.
fn lower_short_circuit_or(session: &mut Session, args: &[WolframForm]) -> AthenaRequest {
    match args {
        [] => AthenaRequest::Term(push_bool(session, false)),
        [only] => lower_request(session, only),
        [first, rest @ ..] => AthenaRequest::Control(ControlPlan::Branch {
            condition: lower_wexpr(session, first),
            then_branch: Box::new(AthenaRequest::Term(push_bool(session, true))),
            else_branch: Some(Box::new(lower_short_circuit_or(session, rest))),
        }),
    }
}

fn push_binding_defines(session: &mut Session, bindings: &WolframForm, steps: &mut Vec<AthenaRequest>) {
    let items: Option<Vec<&WolframForm>> = match bindings {
        WolframForm::List(items) => Some(items.iter().collect()),
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "List") => {
            Some(args.iter().collect())
        }
        _ => None,
    };
    let Some(items) = items
    else {
        return;
    };
    for item in items {
        if let WolframForm::Call { head, args } = item {
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Set") {
                if let [lhs, rhs] = args.as_slice() {
                    if let Some(symbol) = symbol_of(session, lhs) {
                        let value = lower_wexpr(session, rhs);
                        steps.push(AthenaRequest::Command(SessionCommand::Define {
                            symbol,
                            value,
                            kind: BindingKind::Session,
                            evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                        }));
                    }
                }
            }
        }
    }
}

/// Mathematica `Module`: rename locals to fresh `name$n`, then LocalScope.
///
/// Bare `Module[{x}, x]` must not read session OwnValues of `x`.
fn lower_module(session: &mut Session, bindings: &WolframForm, body: &WolframForm) -> AthenaRequest {
    let items = match list_items(bindings) {
        Some(items) => items,
        None => {
            return AthenaRequest::Term(lower_wexpr(
                session,
                &WolframForm::call("Module", vec![bindings.clone(), body.clone()]),
            ));
        }
    };

    let mut renames: Vec<(String, String)> = Vec::new();
    let mut inits: Vec<(String, WolframForm)> = Vec::new();
    for item in items {
        match item {
            WolframForm::Atom(WolframAtom::Symbol(name)) => {
                renames.push((name.clone(), alloc_module_local(session, name)));
            }
            WolframForm::Call { head, args }
                if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Set") =>
            {
                if let [lhs, rhs] = args.as_slice() {
                    if let WolframForm::Atom(WolframAtom::Symbol(name)) = lhs {
                        renames.push((name.clone(), alloc_module_local(session, name)));
                        inits.push((name.clone(), rhs.clone()));
                    }
                }
            }
            _ => {}
        }
    }

    let mut steps = Vec::new();
    for (name, rhs) in &inits {
        let Some((_, fresh)) = renames.iter().find(|(n, _)| n == name)
        else {
            continue;
        };
        let rhs_r = rename_symbols(rhs, &renames);
        let symbol = session.arena.symbols_mut().intern(fresh);
        let value = lower_wexpr(session, &rhs_r);
        steps.push(AthenaRequest::Command(SessionCommand::Define {
            symbol,
            value,
            kind: BindingKind::Lexical,
            evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
        }));
    }
    let body_r = rename_symbols(body, &renames);
    steps.push(lower_request(session, &body_r));
    AthenaRequest::Control(ControlPlan::LocalScope {
        body: Box::new(AthenaRequest::Control(ControlPlan::Sequence { steps })),
    })
}

fn alloc_module_local(session: &mut Session, base: &str) -> String {
    session.module_counter = session.module_counter.saturating_add(1);
    format!("{base}${}", session.module_counter)
}

/// Mathematica `Block`: dynamic shadowing under [`ControlPlan::DynamicScope`].
///
/// Bare `Block[{x}, x]` clears Own for `x` inside the scope (restored on exit).
/// `Block[{x = v}, …]` defines dynamically and restores the prior Own.
fn lower_block(session: &mut Session, bindings: &WolframForm, body: &WolframForm) -> AthenaRequest {
    let items = match list_items(bindings) {
        Some(items) => items,
        None => {
            return AthenaRequest::Term(lower_wexpr(
                session,
                &WolframForm::call("Block", vec![bindings.clone(), body.clone()]),
            ));
        }
    };

    let mut steps = Vec::new();
    for item in items {
        match item {
            WolframForm::Atom(WolframAtom::Symbol(name)) => {
                let symbol = session.arena.symbols_mut().intern(name);
                steps.push(AthenaRequest::Command(SessionCommand::ClearDefinition { symbol }));
            }
            WolframForm::Call { head, args }
                if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Set") =>
            {
                if let [lhs, rhs] = args.as_slice() {
                    if let Some(symbol) = symbol_of(session, lhs) {
                        let value = lower_wexpr(session, rhs);
                        steps.push(AthenaRequest::Command(SessionCommand::Define {
                            symbol,
                            value,
                            kind: BindingKind::Dynamic,
                            evaluation: BindingEvaluationPolicy::EvaluateBeforeStore,
                        }));
                    }
                }
            }
            _ => {}
        }
    }
    steps.push(lower_request(session, body));
    AthenaRequest::Control(ControlPlan::DynamicScope {
        body: Box::new(AthenaRequest::Control(ControlPlan::Sequence { steps })),
    })
}

/// Mathematica `With`: simultaneous lexical replacement of locals by RHS forms.
///
/// RHS are not bound into the session. `With[{x=1,y=x},y]` substitutes `y` → `x`
/// using the outer `x` (not the With local), matching simultaneous semantics.
fn lower_with(session: &mut Session, bindings: &WolframForm, body: &WolframForm) -> AthenaRequest {
    let items = match list_items(bindings) {
        Some(items) => items,
        None => {
            return AthenaRequest::Term(lower_wexpr(
                session,
                &WolframForm::call("With", vec![bindings.clone(), body.clone()]),
            ));
        }
    };

    let mut subst: Vec<(String, WolframForm)> = Vec::new();
    for item in items {
        if let WolframForm::Call { head, args } = item {
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Set") {
                if let [lhs, rhs] = args.as_slice() {
                    if let WolframForm::Atom(WolframAtom::Symbol(name)) = lhs {
                        subst.push((name.clone(), rhs.clone()));
                    }
                }
            }
        }
    }
    let body_r = substitute_with_locals(body, &subst);
    lower_request(session, &body_r)
}

fn substitute_with_locals(form: &WolframForm, subst: &[(String, WolframForm)]) -> WolframForm {
    match form {
        WolframForm::Atom(WolframAtom::Symbol(name)) => {
            if let Some((_, rhs)) = subst.iter().find(|(n, _)| n == name) {
                rhs.clone()
            }
            else {
                form.clone()
            }
        }
        WolframForm::Atom(_) => form.clone(),
        WolframForm::List(items) => WolframForm::List(items.iter().map(|i| substitute_with_locals(i, subst)).collect()),
        WolframForm::Call { head, args } => WolframForm::Call {
            head: Box::new(substitute_with_locals(head, subst)),
            args: args.iter().map(|a| substitute_with_locals(a, subst)).collect(),
        },
    }
}

fn rename_symbols(form: &WolframForm, renames: &[(String, String)]) -> WolframForm {
    match form {
        WolframForm::Atom(WolframAtom::Symbol(name)) => {
            if let Some((_, fresh)) = renames.iter().find(|(n, _)| n == name) {
                WolframForm::symbol(fresh.clone())
            }
            else {
                form.clone()
            }
        }
        WolframForm::Atom(_) => form.clone(),
        WolframForm::List(items) => WolframForm::List(items.iter().map(|i| rename_symbols(i, renames)).collect()),
        WolframForm::Call { head, args } => WolframForm::Call {
            head: Box::new(rename_symbols(head, renames)),
            args: args.iter().map(|a| rename_symbols(a, renames)).collect(),
        },
    }
}

fn symbol_of(session: &mut Session, w: &WolframForm) -> Option<SymbolId> {
    match w {
        WolframForm::Atom(WolframAtom::Symbol(name)) => Some(session.arena.symbols_mut().intern(name)),
        _ => None,
    }
}

fn calculus_goal(request: CalculusRequest) -> AthenaRequest {
    AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::Calculus(request)))
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

fn list_items(w: &WolframForm) -> Option<&[WolframForm]> {
    match w {
        WolframForm::List(items) => Some(items.as_slice()),
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "List") => {
            Some(args.as_slice())
        }
        _ => None,
    }
}

fn symbol_list(session: &mut Session, w: &WolframForm) -> Option<Vec<SymbolId>> {
    let items = list_items(w)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(symbol_of(session, item)?);
    }
    Some(out)
}

fn term_list(session: &mut Session, w: &WolframForm) -> Option<Vec<TermId>> {
    let items = list_items(w)?;
    Some(items.iter().map(|item| lower_wexpr(session, item)).collect())
}

/// Unwrap a single held argument from `Hold` / `HoldForm` / `HoldComplete`.
fn unwrap_hold_form(w: &WolframForm) -> Option<&WolframForm> {
    match w {
        WolframForm::Call { head, args } => {
            let name = match head.as_ref() {
                WolframForm::Atom(WolframAtom::Symbol(s)) => s.as_str(),
                _ => return None,
            };
            if matches!(name, "Hold" | "HoldForm" | "HoldComplete") {
                if let [inner] = args.as_slice() {
                    return Some(inner);
                }
            }
            None
        }
        _ => None,
    }
}

fn derivative_spec(session: &mut Session, spec: &WolframForm) -> Option<(SymbolId, DerivativeOrder)> {
    let items = list_items(spec)?;
    match items {
        [var, order] => {
            let variable = symbol_of(session, var)?;
            let n = match order {
                WolframForm::Atom(WolframAtom::Number(n)) => n.as_exact_integer()?,
                _ => return None,
            };
            if n <= 0 {
                return None;
            }
            let order = if n == 1 { DerivativeOrder::First } else { DerivativeOrder::Repeated(n as u32) };
            Some((variable, order))
        }
        _ => None,
    }
}

fn definite_integral_spec(session: &mut Session, spec: &WolframForm) -> Option<(SymbolId, TermId, TermId)> {
    let items = list_items(spec)?;
    match items {
        [var, lower, upper] => {
            let variable = symbol_of(session, var)?;
            let lower = lower_wexpr(session, lower);
            let upper = lower_wexpr(session, upper);
            Some((variable, lower, upper))
        }
        _ => None,
    }
}

fn limit_rule(session: &mut Session, rule: &WolframForm) -> Option<(SymbolId, LimitApproach, LimitDirection)> {
    let WolframForm::Call { head, args } = rule
    else {
        return None;
    };
    if !matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Rule" || s == "RuleDelayed") {
        return None;
    }
    let [var, point] = args.as_slice()
    else {
        return None;
    };
    let variable = symbol_of(session, var)?;
    let approach = match point {
        WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Infinity" => LimitApproach::PositiveInfinity,
        WolframForm::Call { head, args }
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "DirectedInfinity")
                && args.len() == 1
                && matches!(&args[0], WolframForm::Atom(WolframAtom::Number(n)) if n.as_exact_integer() == Some(1)) =>
        {
            LimitApproach::PositiveInfinity
        }
        WolframForm::Call { head, args }
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "DirectedInfinity")
                && args.len() == 1
                && matches!(&args[0], WolframForm::Atom(WolframAtom::Number(n)) if n.as_exact_integer() == Some(-1)) =>
        {
            LimitApproach::NegativeInfinity
        }
        WolframForm::Call { head, args }
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Minus" || s == "Negate")
                && args.len() == 1
                && matches!(&args[0], WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Infinity") =>
        {
            LimitApproach::NegativeInfinity
        }
        other => LimitApproach::Finite(lower_wexpr(session, other)),
    };
    Some((variable, approach, LimitDirection::TwoSided))
}

/// `Series` iterator `{var, center, order}`.
fn series_spec(session: &mut Session, spec: &WolframForm) -> Option<(SymbolId, TermId, u32)> {
    let items = list_items(spec)?;
    match items {
        [var, center, order] => {
            let variable = symbol_of(session, var)?;
            let center = lower_wexpr(session, center);
            let n = exact_i64(order)?;
            if n < 0 {
                return None;
            }
            Some((variable, center, n as u32))
        }
        _ => None,
    }
}

/// `Residue` iterator `{var, point}`.
fn residue_spec(session: &mut Session, spec: &WolframForm) -> Option<(SymbolId, TermId)> {
    let items = list_items(spec)?;
    match items {
        [var, point] => Some((symbol_of(session, var)?, lower_wexpr(session, point))),
        _ => None,
    }
}

fn extract_table_binder(session: &mut Session, iter: &WolframForm) -> Option<TermId> {
    match iter {
        WolframForm::List(items) if !items.is_empty() => Some(lower_wexpr(session, &items[0])),
        WolframForm::Call { head, args }
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "List") && !args.is_empty() =>
        {
            Some(lower_wexpr(session, &args[0]))
        }
        _ => None,
    }
}

/// `Do` iterator `{n}` / `{i, n}` / `{i, a, b}` → counted-loop variable + value list.
fn do_loop_parts(session: &mut Session, iter: &WolframForm) -> Option<(TermId, TermId)> {
    let items = list_items(iter)?;
    match items {
        [n] => {
            let count = exact_i64(n)?;
            if !(0..=10_000).contains(&count) {
                return None;
            }
            let variable = push_symbol_name(session, "$do");
            let vals: Vec<TermId> = (1..=count).map(|i| push_int(session, i)).collect();
            Some((variable, push_list(session, vals)))
        }
        [_binder, ..] => {
            let variable = extract_table_binder(session, iter)?;
            let iterator = normalize_table_range(session, iter);
            Some((variable, iterator))
        }
        _ => None,
    }
}

/// `Table` iterator `{i, n}` / `{i, a, b}` → ordered collection of values (binder excluded).
fn normalize_table_range(session: &mut Session, iter: &WolframForm) -> TermId {
    let items: Option<Vec<&WolframForm>> = match iter {
        WolframForm::List(items) => Some(items.iter().collect()),
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "List") => {
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
                    }
                    else {
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

fn lower_span_as_range(session: &mut Session, args: &[WolframForm]) -> TermId {
    let arg_ids: Vec<TermId> = args.iter().map(|a| lower_wexpr(session, a)).collect();
    push_semantic(session, SemanticOperator::Range, arg_ids)
}

/// Rewrite pure `Function[body]` with `Slot` into `Function[var, body]` (Living `14`).
fn lower_function(session: &mut Session, args: &[WolframForm]) -> TermId {
    match args {
        [body] => {
            let max_slot = max_slot_index(body).unwrap_or(0);
            if max_slot == 1 {
                let binder_name = "$slot1";
                let rewritten = replace_slots(body, binder_name);
                let binder = push_symbol_name(session, binder_name);
                let body_id = lower_wexpr(session, &rewritten);
                return push_semantic(session, SemanticOperator::Function, vec![binder, body_id]);
            }
            // No slots or multi-slot: keep structural Function[body] (multi-slot later).
            let body_id = lower_wexpr(session, body);
            push_semantic(session, SemanticOperator::Function, vec![body_id])
        }
        [var, body] => {
            let var_id = lower_wexpr(session, var);
            let body_id = lower_wexpr(session, body);
            push_semantic(session, SemanticOperator::Function, vec![var_id, body_id])
        }
        other => {
            let arg_ids: Vec<TermId> = other.iter().map(|a| lower_wexpr(session, a)).collect();
            push_semantic(session, SemanticOperator::Function, arg_ids)
        }
    }
}

fn max_slot_index(w: &WolframForm) -> Option<i64> {
    match w {
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Slot") => {
            exact_i64(args.first()?)
        }
        WolframForm::Call { head, args } => {
            let mut max: Option<i64> = max_slot_index(head);
            for a in args {
                max = match (max, max_slot_index(a)) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                };
            }
            max
        }
        WolframForm::List(items) => {
            let mut max: Option<i64> = None;
            for a in items {
                max = match (max, max_slot_index(a)) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                };
            }
            max
        }
        _ => None,
    }
}

fn replace_slots(w: &WolframForm, binder: &str) -> WolframForm {
    match w {
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Slot") => {
            WolframForm::Atom(WolframAtom::Symbol(binder.to_string()))
        }
        WolframForm::Call { head, args } => WolframForm::Call {
            head: Box::new(replace_slots(head, binder)),
            args: args.iter().map(|a| replace_slots(a, binder)).collect(),
        },
        WolframForm::List(items) => WolframForm::List(items.iter().map(|a| replace_slots(a, binder)).collect()),
        other => other.clone(),
    }
}

fn exact_i64(w: &WolframForm) -> Option<i64> {
    match w {
        WolframForm::Atom(WolframAtom::Number(n)) => n.as_exact_integer(),
        _ => None,
    }
}

/// Mathematica pattern Form → neutral [`TermPattern`] (Living `14`).
fn wexpr_to_term_pattern(session: &mut Session, w: &WolframForm) -> Option<TermPattern> {
    match w {
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Blank") => {
            match args.as_slice() {
                [] => Some(TermPattern::Any),
                [WolframForm::Atom(WolframAtom::Symbol(ty))] if ty == "Integer" => Some(TermPattern::Constrained {
                    pattern: Box::new(TermPattern::Any),
                    constraint: PatternConstraint::ValueType(ValueTypeId::ExactInteger),
                }),
                _ => None,
            }
        }
        WolframForm::Call { head, args }
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Pattern") && args.len() == 2 =>
        {
            let name = match &args[0] {
                WolframForm::Atom(WolframAtom::Symbol(s)) => session.arena.symbols_mut().intern(s),
                _ => return None,
            };
            let inner = wexpr_to_term_pattern(session, &args[1])?;
            Some(TermPattern::Bind { name, inner: Box::new(inner) })
        }
        other => Some(TermPattern::Exact(lower_wexpr(session, other))),
    }
}

fn index_spec_of(w: &WolframForm) -> Option<IndexSpec> {
    match w {
        WolframForm::Atom(WolframAtom::Symbol(s)) if s == "All" => Some(IndexSpec::All),
        WolframForm::Atom(WolframAtom::Number(n)) => n.as_exact_integer().map(|i| IndexSpec::Scalar(IntegerIndex(i))),
        WolframForm::Call { head, args } if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Span" || s == "Range") => {
            match args.as_slice() {
                [start, end] => Some(IndexSpec::Range {
                    start: IntegerIndex(exact_i64(start)?),
                    end: IntegerIndex(exact_i64(end)?),
                    step: 1,
                }),
                [start, end, step] => Some(IndexSpec::Range {
                    start: IntegerIndex(exact_i64(start)?),
                    end: IntegerIndex(exact_i64(end)?),
                    step: exact_i64(step)?,
                }),
                _ => None,
            }
        }
        WolframForm::Call { head, args }
            if matches!(head.as_ref(), WolframForm::Atom(WolframAtom::Symbol(s)) if s == "Plus")
                && args.len() == 2
                && matches!(&args[0], WolframForm::Atom(WolframAtom::Symbol(s)) if s == "end") =>
        {
            // MATLAB-style `end+k` sometimes arrives via other dialects; keep for shared helpers.
            let off = exact_i64(&args[1])?;
            Some(IndexSpec::EndRelative(IntegerOffset(off)))
        }
        WolframForm::Atom(WolframAtom::Symbol(s)) if s == "end" => Some(IndexSpec::EndRelative(IntegerOffset(0))),
        _ => None,
    }
}

/// Session arena [`TermId`] → structural `WolframForm`.
pub fn wexpr_from_session(session: &Session, id: TermId) -> WolframForm {
    match session.arena.get(id) {
        Some(TermNode::Atom(Atom::Number(n))) => WolframForm::Atom(WolframAtom::Number(clone_number(n))),
        Some(TermNode::Atom(Atom::String(s))) => WolframForm::Atom(WolframAtom::String(s.clone())),
        Some(TermNode::Atom(Atom::Boolean(true))) => WolframForm::Atom(WolframAtom::Symbol("True".into())),
        Some(TermNode::Atom(Atom::Boolean(false))) => WolframForm::Atom(WolframAtom::Symbol("False".into())),
        Some(TermNode::Atom(Atom::Null)) => WolframForm::Atom(WolframAtom::Symbol("Null".into())),
        Some(TermNode::Atom(Atom::Constant(MathematicalConstant::Pi))) => WolframForm::Atom(WolframAtom::Symbol("Pi".into())),
        Some(TermNode::Atom(Atom::Constant(MathematicalConstant::EulerNumber))) => {
            WolframForm::Atom(WolframAtom::Symbol("E".into()))
        }
        Some(TermNode::Atom(Atom::Symbol(sym))) => {
            let name = session.arena.symbols().resolve(*sym).unwrap_or("").to_string();
            WolframForm::Atom(WolframAtom::Symbol(name))
        }
        Some(TermNode::Collection { elements: items, .. }) => {
            WolframForm::List(items.iter().map(|i| wexpr_from_session(session, *i)).collect())
        }
        Some(TermNode::Application { head: op, arguments: args }) => {
            let head_name = match *op {
                ApplicationHead::Semantic(SemanticOperator::ApplyHead) if !args.is_empty() => {
                    let head = wexpr_from_session(session, args[0]);
                    let call_args: Vec<WolframForm> = args[1..].iter().map(|a| wexpr_from_session(session, *a)).collect();
                    return WolframForm::Call { head: Box::new(head), args: call_args };
                }
                ApplicationHead::Semantic(sem) => semantic_to_surface(sem).to_string(),
                ApplicationHead::Extension(id) => {
                    let name = session.extensions.display_name(id).unwrap_or("?").to_string();
                    if name == "Application" && !args.is_empty() {
                        let head = wexpr_from_session(session, args[0]);
                        let call_args: Vec<WolframForm> = args[1..].iter().map(|a| wexpr_from_session(session, *a)).collect();
                        return WolframForm::Call { head: Box::new(head), args: call_args };
                    }
                    name
                }
            };
            WolframForm::Call {
                head: Box::new(WolframForm::Atom(WolframAtom::Symbol(head_name))),
                args: args.iter().map(|a| wexpr_from_session(session, *a)).collect(),
            }
        }
        None => WolframForm::Atom(WolframAtom::Symbol(format!("TermId({})", id.0))),
    }
}
