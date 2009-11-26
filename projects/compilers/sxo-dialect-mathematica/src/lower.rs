//! Lower Mathematica Form ([`WExpr`]) into Athena session terms / requests (Living `27`).

use athena::{
    api::{AthenaRequest, ControlPlan, DomainGoal, SessionCommand},
    domains::{
        calculus::{CalculusRequest, DerivativeOrder, LimitApproach, LimitDirection},
        DomainRequest,
    },
    ir::{ApplicationHead, Atom, MathematicalConstant, SemanticOperator, TermNode, UnaryFunction},
    reasoning::trs::{PatternConstraint, TermPattern},
    runtime::values::arena::{
        push_bool, push_constant, push_extension, push_int, push_list, push_null, push_semantic, push_symbol_name,
    },
    runtime::values::numeric_clone::clone_number,
    types::{
        AssumptionSet, BindingEvaluationPolicy, BindingKind, IndexSpec, IntegerIndex, IntegerOffset, SymbolId,
        TermId, ValueTypeId,
    },
    Session,
};

use crate::form::{WAtom, WExpr};

/// Map a Mathematica surface head to a closed [`SemanticOperator`] when known.
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
    } else {
        let op = session.operators.intern(name);
        push_extension(session, op, args)
    }
}

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
            WAtom::Symbol(s) if s == "Pi" => push_constant(session, MathematicalConstant::Pi),
            WAtom::Symbol(s) if s == "E" => push_constant(session, MathematicalConstant::EulerNumber),
            WAtom::Symbol(s) => push_symbol_name(session, s),
        },
        WExpr::List(items) => {
            let ids: Vec<TermId> = items.iter().map(|i| lower_wexpr(session, i)).collect();
            push_list(session, ids)
        }
        WExpr::Call { head, args } => match head.as_ref() {
            WExpr::Atom(WAtom::Symbol(name)) if name == "Function" => lower_function(session, args),
            WExpr::Atom(WAtom::Symbol(name)) if name == "Span" => lower_span_as_range(session, args),
            WExpr::Atom(WAtom::Symbol(name)) if name == "Apply" || name == "Map" => {
                let mut arg_ids = Vec::with_capacity(args.len());
                for (i, a) in args.iter().enumerate() {
                    if i == 0 {
                        arg_ids.push(lower_operator_value(session, a));
                    } else {
                        arg_ids.push(lower_wexpr(session, a));
                    }
                }
                push_surface_call(session, name, arg_ids)
            }
            WExpr::Atom(WAtom::Symbol(name)) => {
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
fn lower_operator_value(session: &mut Session, w: &WExpr) -> TermId {
    match w {
        WExpr::Atom(WAtom::Symbol(name)) => {
            if let Some(op) = surface_to_semantic(name) {
                push_semantic(session, op, Vec::new())
            } else {
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
pub fn lower_request(session: &mut Session, w: &WExpr) -> AthenaRequest {
    match w {
        WExpr::Call { head, args } => match head.as_ref() {
            WExpr::Atom(WAtom::Symbol(name)) => match (name.as_str(), args.as_slice()) {
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
                ("CompoundExpression", args) => {
                    let steps: Vec<AthenaRequest> =
                        args.iter().map(|a| lower_request(session, a)).collect();
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
                ("With" | "Module" | "Block", [bindings, body]) => {
                    let mut steps = Vec::new();
                    push_binding_defines(session, bindings, &mut steps);
                    steps.push(lower_request(session, body));
                    return AthenaRequest::Control(ControlPlan::LocalScope {
                        body: Box::new(AthenaRequest::Control(ControlPlan::Sequence { steps })),
                    });
                }
                ("Part", args) if args.len() >= 2 => {
                    if let Some(axes) = args[1..].iter().map(index_spec_of).collect::<Option<Vec<_>>>() {
                        return AthenaRequest::Control(ControlPlan::Index {
                            target: lower_wexpr(session, &args[0]),
                            axes,
                        });
                    }
                }
                ("MatchQ", [expr, pat]) => {
                    if let Some(pattern) = wexpr_to_term_pattern(session, pat) {
                        return AthenaRequest::Control(ControlPlan::Match {
                            target: lower_wexpr(session, expr),
                            pattern,
                        });
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
            },
            _ => {}
        },
        _ => {}
    }
    AthenaRequest::Term(lower_wexpr(session, w))
}

fn push_binding_defines(session: &mut Session, bindings: &WExpr, steps: &mut Vec<AthenaRequest>) {
    let items: Option<Vec<&WExpr>> = match bindings {
        WExpr::List(items) => Some(items.iter().collect()),
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "List") =>
        {
            Some(args.iter().collect())
        }
        _ => None,
    };
    let Some(items) = items else {
        return;
    };
    for item in items {
        if let WExpr::Call { head, args } = item {
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Set") {
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

fn symbol_of(session: &mut Session, w: &WExpr) -> Option<SymbolId> {
    match w {
        WExpr::Atom(WAtom::Symbol(name)) => Some(session.arena.symbols_mut().intern(name)),
        _ => None,
    }
}

fn calculus_goal(request: CalculusRequest) -> AthenaRequest {
    AthenaRequest::Goal(DomainGoal::Dispatch(DomainRequest::Calculus(request)))
}

fn list_items(w: &WExpr) -> Option<&[WExpr]> {
    match w {
        WExpr::List(items) => Some(items.as_slice()),
        WExpr::Call { head, args } if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "List") => {
            Some(args.as_slice())
        }
        _ => None,
    }
}

fn derivative_spec(session: &mut Session, spec: &WExpr) -> Option<(SymbolId, DerivativeOrder)> {
    let items = list_items(spec)?;
    match items {
        [var, order] => {
            let variable = symbol_of(session, var)?;
            let n = match order {
                WExpr::Atom(WAtom::Number(n)) => n.as_exact_integer()?,
                _ => return None,
            };
            if n <= 0 {
                return None;
            }
            let order = if n == 1 {
                DerivativeOrder::First
            } else {
                DerivativeOrder::Repeated(n as u32)
            };
            Some((variable, order))
        }
        _ => None,
    }
}

fn definite_integral_spec(session: &mut Session, spec: &WExpr) -> Option<(SymbolId, TermId, TermId)> {
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

fn limit_rule(session: &mut Session, rule: &WExpr) -> Option<(SymbolId, LimitApproach, LimitDirection)> {
    let WExpr::Call { head, args } = rule else {
        return None;
    };
    if !matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Rule" || s == "RuleDelayed") {
        return None;
    }
    let [var, point] = args.as_slice() else {
        return None;
    };
    let variable = symbol_of(session, var)?;
    let approach = match point {
        WExpr::Atom(WAtom::Symbol(s)) if s == "Infinity" => LimitApproach::PositiveInfinity,
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "DirectedInfinity")
                && args.len() == 1
                && matches!(&args[0], WExpr::Atom(WAtom::Number(n)) if n.as_exact_integer() == Some(1)) =>
        {
            LimitApproach::PositiveInfinity
        }
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "DirectedInfinity")
                && args.len() == 1
                && matches!(&args[0], WExpr::Atom(WAtom::Number(n)) if n.as_exact_integer() == Some(-1)) =>
        {
            LimitApproach::NegativeInfinity
        }
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Minus" || s == "Negate")
                && args.len() == 1
                && matches!(&args[0], WExpr::Atom(WAtom::Symbol(s)) if s == "Infinity") =>
        {
            LimitApproach::NegativeInfinity
        }
        other => LimitApproach::Finite(lower_wexpr(session, other)),
    };
    Some((variable, approach, LimitDirection::TwoSided))
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
    push_semantic(session, SemanticOperator::Range, arg_ids)
}

/// Rewrite pure `Function[body]` with `Slot` into `Function[var, body]` (Living `27`).
fn lower_function(session: &mut Session, args: &[WExpr]) -> TermId {
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

fn max_slot_index(w: &WExpr) -> Option<i64> {
    match w {
        WExpr::Call { head, args } if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Slot") => {
            exact_i64(args.first()?)
        }
        WExpr::Call { head, args } => {
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
        WExpr::List(items) => {
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

fn replace_slots(w: &WExpr, binder: &str) -> WExpr {
    match w {
        WExpr::Call { head, args } if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Slot") => {
            WExpr::Atom(WAtom::Symbol(binder.to_string()))
        }
        WExpr::Call { head, args } => WExpr::Call {
            head: Box::new(replace_slots(head, binder)),
            args: args.iter().map(|a| replace_slots(a, binder)).collect(),
        },
        WExpr::List(items) => WExpr::List(items.iter().map(|a| replace_slots(a, binder)).collect()),
        other => other.clone(),
    }
}

fn exact_i64(w: &WExpr) -> Option<i64> {
    match w {
        WExpr::Atom(WAtom::Number(n)) => n.as_exact_integer(),
        _ => None,
    }
}

/// Mathematica pattern Form → neutral [`TermPattern`] (Living `27`).
fn wexpr_to_term_pattern(session: &mut Session, w: &WExpr) -> Option<TermPattern> {
    match w {
        WExpr::Call { head, args } if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Blank") => {
            match args.as_slice() {
                [] => Some(TermPattern::Any),
                [WExpr::Atom(WAtom::Symbol(ty))] if ty == "Integer" => Some(TermPattern::Constrained {
                    pattern: Box::new(TermPattern::Any),
                    constraint: PatternConstraint::ValueType(ValueTypeId::ExactInteger),
                }),
                _ => None,
            }
        }
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Pattern") && args.len() == 2 =>
        {
            let name = match &args[0] {
                WExpr::Atom(WAtom::Symbol(s)) => session.arena.symbols_mut().intern(s),
                _ => return None,
            };
            let inner = wexpr_to_term_pattern(session, &args[1])?;
            Some(TermPattern::Bind {
                name,
                inner: Box::new(inner),
            })
        }
        other => Some(TermPattern::Exact(lower_wexpr(session, other))),
    }
}

fn index_spec_of(w: &WExpr) -> Option<IndexSpec> {
    match w {
        WExpr::Atom(WAtom::Symbol(s)) if s == "All" => Some(IndexSpec::All),
        WExpr::Atom(WAtom::Number(n)) => n.as_exact_integer().map(|i| IndexSpec::Scalar(IntegerIndex(i))),
        WExpr::Call { head, args } if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Span" || s == "Range") => {
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
        WExpr::Call { head, args }
            if matches!(head.as_ref(), WExpr::Atom(WAtom::Symbol(s)) if s == "Plus")
                && args.len() == 2
                && matches!(&args[0], WExpr::Atom(WAtom::Symbol(s)) if s == "end") =>
        {
            // MATLAB-style `end+k` sometimes arrives via other dialects; keep for shared helpers.
            let off = exact_i64(&args[1])?;
            Some(IndexSpec::EndRelative(IntegerOffset(off)))
        }
        WExpr::Atom(WAtom::Symbol(s)) if s == "end" => Some(IndexSpec::EndRelative(IntegerOffset(0))),
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
        Some(TermNode::Atom(Atom::Constant(MathematicalConstant::Pi))) => WExpr::Atom(WAtom::Symbol("Pi".into())),
        Some(TermNode::Atom(Atom::Constant(MathematicalConstant::EulerNumber))) => {
            WExpr::Atom(WAtom::Symbol("E".into()))
        }
        Some(TermNode::Atom(Atom::Symbol(sym))) => {
            let name = session.arena.symbols().resolve(*sym).unwrap_or("").to_string();
            WExpr::Atom(WAtom::Symbol(name))
        }
        Some(TermNode::Collection { elements: items, .. }) => {
            WExpr::List(items.iter().map(|i| wexpr_from_session(session, *i)).collect())
        }
        Some(TermNode::Application { head: op, arguments: args }) => {
            let head_name = match *op {
                ApplicationHead::Semantic(SemanticOperator::ApplyHead) if !args.is_empty() => {
                    let head = wexpr_from_session(session, args[0]);
                    let call_args: Vec<WExpr> = args[1..].iter().map(|a| wexpr_from_session(session, *a)).collect();
                    return WExpr::Call {
                        head: Box::new(head),
                        args: call_args,
                    };
                }
                ApplicationHead::Semantic(sem) => semantic_to_surface(sem).to_string(),
                ApplicationHead::Extension(id) => {
                    let name = session.operators.name(id).unwrap_or("?").to_string();
                    if name == "Application" && !args.is_empty() {
                        let head = wexpr_from_session(session, args[0]);
                        let call_args: Vec<WExpr> = args[1..].iter().map(|a| wexpr_from_session(session, *a)).collect();
                        return WExpr::Call {
                            head: Box::new(head),
                            args: call_args,
                        };
                    }
                    name
                }
            };
            WExpr::Call {
                head: Box::new(WExpr::Atom(WAtom::Symbol(head_name))),
                args: args.iter().map(|a| wexpr_from_session(session, *a)).collect(),
            }
        }
        None => WExpr::Atom(WAtom::Symbol(format!("TermId({})", id.0))),
    }
}
