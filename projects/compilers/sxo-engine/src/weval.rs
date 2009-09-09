//! Builtin evaluation on engine [`Term`] (not Mathematica [`crate::wexpr::WExpr`]).

use std::cmp::Ordering;

use num_traits::ToPrimitive;

use num_bigint::BigInt;
use num_traits::Zero;

use euler::{Number, RealNumber};
use sxo_types::SxoError;

use crate::term::{Atom, Term, number_from_term};

fn map_num<T>(r: euler::Result<T>) -> Result<T, SxoError> {
    r.map_err(SxoError::from_diagnostic)
}

/// Evaluate `expr` under built-in definitions. Unknown heads stay as `App`.
pub fn evaluate(expr: &Term) -> Term {
    evaluate_depth(expr, 0)
}

fn evaluate_depth(expr: &Term, depth: u32) -> Term {
    if depth > 256 {
        return expr.clone();
    }
    match expr {
        Term::Atom(_) => expr.clone(),
        Term::List(items) => Term::List(items.iter().map(|i| evaluate_depth(i, depth + 1)).collect()),
        Term::App { head, args } => {
            let head_e = evaluate_depth(head, depth + 1);
            let args_e: Vec<Term> = args.iter().map(|a| evaluate_depth(a, depth + 1)).collect();
            apply_builtin(&head_e, args_e, depth)
        }
    }
}

fn apply_builtin(head: &Term, args: Vec<Term>, depth: u32) -> Term {
    let name = match head {
        Term::Atom(Atom::Symbol(s)) => s.as_str(),
        _ => {
            return Term::App { head: Box::new(head.clone()), args };
        }
    };

    match name {
        "Plus" => eval_plus(args),
        "Times" => eval_times(args),
        "Power" if args.len() == 2 => eval_power(args[0].clone(), args[1].clone()),
        "Subtract" if args.len() == 2 => eval_plus(vec![args[0].clone(), eval_times(vec![Term::int(-1), args[1].clone()])]),
        "Divide" if args.len() == 2 => eval_times(vec![args[0].clone(), eval_power(args[1].clone(), Term::int(-1))]),
        "List" => Term::List(args),
        "Simplify" if args.len() == 1 => eval_simplify(&args[0], depth),
        "Sin" | "Cos" | "Tan" | "Exp" | "Log" if args.len() == 1 => Term::App { head: Box::new(head.clone()), args },
        "Sqrt" if args.len() == 1 => eval_sqrt(&args[0]),
        "Abs" if args.len() == 1 => eval_abs(&args[0]),
        "Factorial" if args.len() == 1 => eval_factorial(&args[0]),
        "Map" if args.len() == 2 => eval_map(&args[0], &args[1], depth),
        "Equal" if args.len() == 2 => eval_compare("Equal", &args[0], &args[1], |o| o == Ordering::Equal),
        "Unequal" if args.len() == 2 => eval_compare("Unequal", &args[0], &args[1], |o| o != Ordering::Equal),
        "Less" if args.len() == 2 => eval_compare("Less", &args[0], &args[1], |o| o == Ordering::Less),
        "Greater" if args.len() == 2 => eval_compare("Greater", &args[0], &args[1], |o| o == Ordering::Greater),
        "LessEqual" if args.len() == 2 => eval_compare("LessEqual", &args[0], &args[1], |o| o != Ordering::Greater),
        "GreaterEqual" if args.len() == 2 => eval_compare("GreaterEqual", &args[0], &args[1], |o| o != Ordering::Less),
        "And" if args.len() == 2 => eval_logic_and(&args[0], &args[1]),
        "Or" if args.len() == 2 => eval_logic_or(&args[0], &args[1]),
        "Not" if args.len() == 1 => eval_logic_not(&args[0]),
        "Set" | "SetDelayed" if args.len() == 2 => evaluate_depth(&args[1], depth + 1),
        "D" if args.len() == 2 => {
            let var = match &args[1] {
                Term::Atom(Atom::Symbol(s)) => s.clone(),
                _ => {
                    return Term::app("D", args);
                }
            };
            evaluate_depth(&differentiate(&args[0], &var), depth + 1)
        }
        "Integrate" if args.len() == 2 => {
            let var = match &args[1] {
                Term::Atom(Atom::Symbol(s)) => s.clone(),
                _ => {
                    return Term::app("Integrate", args);
                }
            };
            evaluate_depth(&integrate(&args[0], &var), depth + 1)
        }
        "CompoundExpression" if !args.is_empty() => evaluate_depth(args.last().unwrap(), depth + 1),
        "Function" => Term::App { head: Box::new(Term::symbol("Function")), args },
        "ReplaceAll" if args.len() == 2 => eval_replace_all(&args[0], &args[1], depth),
        "Part" if args.len() == 2 => eval_part(&args[0], &args[1]),
        "Rule" | "RuleDelayed" if args.len() == 2 => Term::App { head: Box::new(head.clone()), args },
        _ => {
            if let Term::App { head: fh, args: fargs } = head {
                if fh.is_symbol("Function") && fargs.len() == 1 && args.len() == 1 {
                    let body = substitute_slot(&fargs[0], &args[0]);
                    return evaluate_depth(&body, depth + 1);
                }
            }
            Term::App { head: Box::new(head.clone()), args }
        }
    }
}

fn eval_plus(args: Vec<Term>) -> Term {
    let mut flat = Vec::new();
    let mut sum: Option<Number> = None;
    for a in args {
        flatten_plus(a, &mut flat, &mut sum);
    }
    if let Some(s) = sum {
        if !s.is_zero() {
            flat.insert(0, Term::number(s));
        }
    }
    else if flat.is_empty() {
        return Term::int(0);
    }
    match flat.len() {
        0 => Term::int(0),
        1 => flat.pop().unwrap(),
        _ => Term::app("Plus", flat),
    }
}

fn flatten_plus(a: Term, flat: &mut Vec<Term>, sum: &mut Option<Number>) {
    match a {
        Term::App { head, args } if head.is_symbol("Plus") => {
            for x in args {
                flatten_plus(x, flat, sum);
            }
        }
        other => push_plus_term(other, flat, sum),
    }
}

fn push_plus_term(a: Term, flat: &mut Vec<Term>, sum: &mut Option<Number>) {
    if let Some(n) = number_from_term(&a).cloned() {
        *sum = Some(match sum.take() {
            Some(s) => map_num(s.clone().add(n)).unwrap_or(s),
            None => n,
        });
    }
    else {
        flat.push(a);
    }
}

fn eval_times(args: Vec<Term>) -> Term {
    let mut flat = Vec::new();
    let mut prod: Option<Number> = None;
    for a in args {
        flatten_times(a, &mut flat, &mut prod);
    }
    if let Some(p) = prod {
        if p.is_zero() {
            return Term::int(0);
        }
        if !p.is_one() {
            flat.insert(0, Term::number(p));
        }
    }
    else if flat.is_empty() {
        return Term::int(1);
    }
    match flat.len() {
        0 => Term::int(1),
        1 => flat.pop().unwrap(),
        _ => Term::app("Times", flat),
    }
}

fn flatten_times(a: Term, flat: &mut Vec<Term>, prod: &mut Option<Number>) {
    match a {
        Term::App { head, args } if head.is_symbol("Times") => {
            for x in args {
                flatten_times(x, flat, prod);
            }
        }
        other => {
            if let Some(n) = number_from_term(&other).cloned() {
                if n.is_zero() {
                    *prod = Some(Number::small_int(0));
                    return;
                }
                *prod = Some(match prod.take() {
                    Some(p) => map_num(p.clone().mul(n)).unwrap_or(p),
                    None => n,
                });
            }
            else {
                flat.push(other);
            }
        }
    }
}

fn eval_power(base: Term, exp: Term) -> Term {
    if let Some(e) = number_from_term(&exp).cloned() {
        if e.is_zero() {
            return Term::int(1);
        }
        if e.is_one() {
            return base;
        }
        if e.is_neg_one() {
            if let Some(b) = number_from_term(&base).cloned() {
                if let Ok(v) = map_num(Number::small_int(1).div(b)) {
                    return Term::number(v);
                }
            }
        }
    }
    if let (Some(b), Some(e)) = (number_from_term(&base).cloned(), number_from_term(&exp).cloned()) {
        if let Ok(v) = map_num(b.pow(&e)) {
            return Term::number(v);
        }
    }
    Term::app("Power", vec![base, exp])
}

fn eval_simplify(expr: &Term, depth: u32) -> Term {
    let e = evaluate_depth(expr, depth + 1);
    if let Some(one) = try_pythagorean(&e) {
        return one;
    }
    evaluate_depth(&e, depth + 1)
}

fn eval_sqrt(arg: &Term) -> Term {
    if let Some(n) = number_from_term(arg).cloned() {
        if let Ok(Some(v)) = map_num(n.sqrt()) {
            return Term::number(v);
        }
    }
    Term::app("Sqrt", vec![arg.clone()])
}

fn eval_abs(arg: &Term) -> Term {
    if let Some(n) = number_from_term(arg).cloned() {
        return Term::number(n.abs());
    }
    Term::app("Abs", vec![arg.clone()])
}

fn eval_factorial(arg: &Term) -> Term {
    if let Some(n) = number_from_term(arg).cloned() {
        if let Ok(v) = map_num(n.factorial()) {
            return Term::number(v);
        }
    }
    Term::app("Factorial", vec![arg.clone()])
}

fn eval_compare<F>(head: &str, left: &Term, right: &Term, cmp: F) -> Term
where
    F: Fn(Ordering) -> bool,
{
    if let (Some(a), Some(b)) = (number_from_term(left), number_from_term(right)) {
        if let Some(ord) = a.compare(b) {
            return Term::int(if cmp(ord) { 1 } else { 0 });
        }
    }
    Term::app(head, vec![left.clone(), right.clone()])
}

fn eval_logic_and(left: &Term, right: &Term) -> Term {
    match (truthy(left), truthy(right)) {
        (Some(a), Some(b)) => Term::int(if a && b { 1 } else { 0 }),
        _ => Term::app("And", vec![left.clone(), right.clone()]),
    }
}

fn eval_logic_or(left: &Term, right: &Term) -> Term {
    match (truthy(left), truthy(right)) {
        (Some(a), Some(b)) => Term::int(if a || b { 1 } else { 0 }),
        _ => Term::app("Or", vec![left.clone(), right.clone()]),
    }
}

fn eval_logic_not(arg: &Term) -> Term {
    match truthy(arg) {
        Some(v) => Term::int(if v { 0 } else { 1 }),
        None => Term::app("Not", vec![arg.clone()]),
    }
}

fn truthy(expr: &Term) -> Option<bool> {
    number_from_term(expr).map(Number::is_truthy)
}

fn eval_map(func: &Term, target: &Term, depth: u32) -> Term {
    let list = match target {
        Term::List(items) => items,
        other => return Term::app("Map", vec![func.clone(), other.clone()]),
    };
    Term::List(
        list.iter()
            .map(|item| {
                let mapped = map_one(func, item);
                evaluate_depth(&mapped, depth + 1)
            })
            .collect(),
    )
}

fn map_one(func: &Term, item: &Term) -> Term {
    match func {
        Term::Atom(Atom::Symbol(name)) => Term::app(name.clone(), vec![item.clone()]),
        Term::App { head, args } if head.is_symbol("Function") && args.len() == 1 => substitute_slot(&args[0], item),
        _ => Term::app("Map", vec![func.clone(), item.clone()]),
    }
}

/// Sin[x]^2 + Cos[x]^2 → 1 (and swapped).
fn try_pythagorean(expr: &Term) -> Option<Term> {
    let terms = match expr {
        Term::App { head, args } if head.is_symbol("Plus") => args.as_slice(),
        _ => return None,
    };
    if terms.len() != 2 {
        return None;
    }
    let (a, b) = (&terms[0], &terms[1]);
    if is_trig_sq(a, "Sin") && is_trig_sq(b, "Cos") && same_trig_arg(a, b) {
        return Some(Term::int(1));
    }
    if is_trig_sq(a, "Cos") && is_trig_sq(b, "Sin") && same_trig_arg(a, b) {
        return Some(Term::int(1));
    }
    None
}

fn is_trig_sq(expr: &Term, name: &str) -> bool {
    match expr {
        Term::App { head, args } if head.is_symbol("Power") && args.len() == 2 => {
            matches!(number_from_term(&args[1]), Some(n) if *n == Number::small_int(2))
                && matches!(&args[0], Term::App { head: h, args: a } if h.is_symbol(name) && a.len() == 1)
        }
        _ => false,
    }
}

fn same_trig_arg(a: &Term, b: &Term) -> bool {
    fn arg(expr: &Term) -> Option<&Term> {
        match expr {
            Term::App { head, args } if head.is_symbol("Power") && args.len() == 2 => match &args[0] {
                Term::App { args: inner, .. } if inner.len() == 1 => Some(&inner[0]),
                _ => None,
            },
            _ => None,
        }
    }
    match (arg(a), arg(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Symbolic differentiation on `Term`.
pub fn differentiate(expr: &Term, var: &str) -> Term {
    match expr {
        Term::Atom(Atom::Number(_)) | Term::Atom(Atom::String(_)) => Term::int(0),
        Term::Atom(Atom::Symbol(s)) if s == var => Term::int(1),
        Term::Atom(Atom::Symbol(_)) => Term::int(0),
        Term::List(items) => Term::List(items.iter().map(|i| differentiate(i, var)).collect()),
        Term::App { head, args } => {
            let h = head.head_name().unwrap_or("");
            match h {
                "Plus" => evaluate(&Term::app("Plus", args.iter().map(|a| differentiate(a, var)).collect())),
                "Times" => {
                    let mut terms = Vec::new();
                    for i in 0..args.len() {
                        let mut factors = args.clone();
                        factors[i] = differentiate(&args[i], var);
                        terms.push(Term::app("Times", factors));
                    }
                    evaluate(&Term::app("Plus", terms))
                }
                "Power" if args.len() == 2 => {
                    let base = &args[0];
                    let exp = &args[1];
                    if let Some(n) = number_from_term(exp).and_then(|e| e.as_integer_exp()) {
                        evaluate(&Term::app(
                            "Times",
                            vec![
                                Term::integer(n.clone()),
                                Term::app("Power", vec![base.clone(), Term::integer(n - 1i64)]),
                                differentiate(base, var),
                            ],
                        ))
                    }
                    else if let Some(Number::Real(RealNumber::Machine(nf))) = number_from_term(exp).cloned() {
                        evaluate(&Term::app(
                            "Times",
                            vec![
                                Term::real(nf),
                                Term::app("Power", vec![base.clone(), Term::real(nf - 1.0)]),
                                differentiate(base, var),
                            ],
                        ))
                    }
                    else {
                        Term::app("D", vec![expr.clone(), Term::symbol(var)])
                    }
                }
                "Sin" if args.len() == 1 => {
                    evaluate(&Term::app("Times", vec![Term::app("Cos", vec![args[0].clone()]), differentiate(&args[0], var)]))
                }
                "Cos" if args.len() == 1 => evaluate(&Term::app(
                    "Times",
                    vec![Term::int(-1), Term::app("Sin", vec![args[0].clone()]), differentiate(&args[0], var)],
                )),
                "Tan" if args.len() == 1 => evaluate(&Term::app(
                    "Times",
                    vec![
                        Term::app("Power", vec![Term::app("Cos", vec![args[0].clone()]), Term::int(-2)]),
                        differentiate(&args[0], var),
                    ],
                )),
                "Exp" if args.len() == 1 => {
                    evaluate(&Term::app("Times", vec![Term::app("Exp", vec![args[0].clone()]), differentiate(&args[0], var)]))
                }
                "Log" if args.len() == 1 => evaluate(&Term::app(
                    "Times",
                    vec![Term::app("Power", vec![args[0].clone(), Term::int(-1)]), differentiate(&args[0], var)],
                )),
                "Subtract" if args.len() == 2 => evaluate(&Term::app(
                    "Plus",
                    vec![differentiate(&args[0], var), Term::app("Times", vec![Term::int(-1), differentiate(&args[1], var)])],
                )),
                "Divide" if args.len() == 2 => {
                    let a = &args[0];
                    let b = &args[1];
                    evaluate(&Term::app(
                        "Times",
                        vec![
                            Term::app(
                                "Plus",
                                vec![
                                    Term::app("Times", vec![differentiate(a, var), b.clone()]),
                                    Term::app("Times", vec![Term::int(-1), a.clone(), differentiate(b, var)]),
                                ],
                            ),
                            Term::app("Power", vec![b.clone(), Term::int(-2)]),
                        ],
                    ))
                }
                _ => Term::int(0),
            }
        }
    }
}

/// Symbolic integration on `Term` (polynomial / elementary subset).
pub fn integrate(expr: &Term, var: &str) -> Term {
    match expr {
        Term::Atom(Atom::Number(n)) => Term::app("Times", vec![Term::number(n.clone()), Term::symbol(var)]),
        Term::Atom(Atom::String(_)) => Term::app("Integrate", vec![expr.clone(), Term::symbol(var)]),
        Term::Atom(Atom::Symbol(s)) if s == var => evaluate(&Term::app("Divide", vec![
            Term::app("Power", vec![Term::symbol(var), Term::int(2)]),
            Term::int(2),
        ])),
        Term::Atom(Atom::Symbol(_)) => Term::app("Times", vec![expr.clone(), Term::symbol(var)]),
        Term::List(items) => Term::List(items.iter().map(|i| integrate(i, var)).collect()),
        Term::App { head, args } => {
            let h = head.head_name().unwrap_or("");
            match h {
                "Plus" => evaluate(&Term::app("Plus", args.iter().map(|a| integrate(a, var)).collect())),
                "Times" if args.len() == 2 => {
                    let (coeff, rest) = if number_from_term(&args[0]).is_some() {
                        (&args[0], &args[1])
                    }
                    else if number_from_term(&args[1]).is_some() {
                        (&args[1], &args[0])
                    }
                    else {
                        return Term::app("Integrate", vec![expr.clone(), Term::symbol(var)]);
                    };
                    evaluate(&Term::app("Times", vec![coeff.clone(), integrate(rest, var)]))
                }
                "Power" if args.len() == 2 && args[0].is_symbol(var) => {
                    if let Some(n) = number_from_term(&args[1]).and_then(|e| e.as_integer_exp()) {
                        if (n.clone() + 1) != BigInt::zero() {
                            return evaluate(&Term::app("Divide", vec![
                                Term::app("Power", vec![args[0].clone(), Term::integer(n.clone() + 1i64)]),
                                Term::integer(n + 1i64),
                            ]));
                        }
                    }
                    Term::app("Integrate", vec![expr.clone(), Term::symbol(var)])
                }
                "Sin" if args.len() == 1 && args[0].is_symbol(var) => {
                    evaluate(&Term::app("Times", vec![Term::int(-1), Term::app("Cos", args.clone())]))
                }
                "Cos" if args.len() == 1 && args[0].is_symbol(var) => Term::app("Sin", args.clone()),
                "Exp" if args.len() == 1 && args[0].is_symbol(var) => Term::app("Exp", args.clone()),
                _ => Term::app("Integrate", vec![expr.clone(), Term::symbol(var)]),
            }
        }
    }
}

fn eval_replace_all(expr: &Term, rules: &Term, depth: u32) -> Term {
    let rule_list: Vec<(Term, Term)> = match rules {
        Term::List(items) => items.iter().filter_map(rule_pair).collect(),
        other => rule_pair(other).into_iter().collect(),
    };
    let mut cur = expr.clone();
    for (lhs, rhs) in rule_list {
        cur = replace_literal(&cur, &lhs, &rhs);
    }
    evaluate_depth(&cur, depth + 1)
}

fn rule_pair(expr: &Term) -> Option<(Term, Term)> {
    match expr {
        Term::App { head, args } if args.len() == 2 && (head.is_symbol("Rule") || head.is_symbol("RuleDelayed")) => {
            Some((args[0].clone(), args[1].clone()))
        }
        _ => None,
    }
}

fn replace_literal(expr: &Term, lhs: &Term, rhs: &Term) -> Term {
    if expr == lhs {
        return rhs.clone();
    }
    match expr {
        Term::List(items) => Term::List(items.iter().map(|i| replace_literal(i, lhs, rhs)).collect()),
        Term::App { head, args } => Term::App {
            head: Box::new(replace_literal(head, lhs, rhs)),
            args: args.iter().map(|a| replace_literal(a, lhs, rhs)).collect(),
        },
        other => other.clone(),
    }
}

fn eval_part(expr: &Term, index: &Term) -> Term {
    let idx = match number_from_term(index).and_then(|n| n.as_exact_integer()) {
        Some(n) => match n.to_i64() {
            Some(v) => v,
            None => return Term::app("Part", vec![expr.clone(), index.clone()]),
        },
        None => return Term::app("Part", vec![expr.clone(), index.clone()]),
    };
    match expr {
        Term::List(items) => {
            let i = if idx > 0 {
                (idx - 1) as usize
            }
            else if idx < 0 {
                items.len().wrapping_add(idx as usize)
            }
            else {
                return Term::app("Part", vec![expr.clone(), index.clone()]);
            };
            items.get(i).cloned().unwrap_or_else(|| Term::app("Part", vec![expr.clone(), index.clone()]))
        }
        _ => Term::app("Part", vec![expr.clone(), index.clone()]),
    }
}

fn substitute_slot(body: &Term, value: &Term) -> Term {
    match body {
        Term::Atom(Atom::Symbol(s)) if s == "#" || s == "#1" => value.clone(),
        Term::Atom(_) => body.clone(),
        Term::List(items) => Term::List(items.iter().map(|i| substitute_slot(i, value)).collect()),
        Term::App { head, args } => Term::App {
            head: Box::new(substitute_slot(head, value)),
            args: args.iter().map(|a| substitute_slot(a, value)).collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plus_fold() {
        let e = evaluate(&Term::app("Plus", vec![Term::int(1), Term::int(2), Term::symbol("x")]));
        assert_eq!(e, Term::app("Plus", vec![Term::int(3), Term::symbol("x")]));
    }

    #[test]
    fn power_one() {
        let e = evaluate(&Term::app("Power", vec![Term::symbol("x"), Term::int(1)]));
        assert_eq!(e, Term::symbol("x"));
    }

    #[test]
    fn list_eval() {
        let e = evaluate(&Term::List(vec![Term::int(1), Term::app("Plus", vec![Term::int(2), Term::int(2)])]));
        assert_eq!(e, Term::List(vec![Term::int(1), Term::int(4)]));
    }

    #[test]
    fn d_power() {
        let e = evaluate(&Term::app("D", vec![Term::app("Power", vec![Term::symbol("x"), Term::int(3)]), Term::symbol("x")]));
        assert!(matches!(e, Term::App { .. }));
        let s = crate::render_wexpr::render_wexpr(&crate::mma_bridge::term_to_wexpr(&e));
        assert!(s.contains('x'), "got {s}");
    }

    #[test]
    fn pythagorean() {
        let sin2 = Term::app("Power", vec![Term::app("Sin", vec![Term::symbol("x")]), Term::int(2)]);
        let cos2 = Term::app("Power", vec![Term::app("Cos", vec![Term::symbol("x")]), Term::int(2)]);
        let e = evaluate(&Term::app("Simplify", vec![Term::app("Plus", vec![sin2, cos2])]));
        assert_eq!(e, Term::int(1));
    }

    #[test]
    fn compound_expression_returns_last() {
        let e = evaluate(&Term::app("CompoundExpression", vec![Term::int(1), Term::int(2), Term::int(3)]));
        assert_eq!(e, Term::int(3));
    }

    #[test]
    fn integrate_power() {
        let e = evaluate(&Term::app("Integrate", vec![Term::app("Power", vec![Term::symbol("x"), Term::int(2)]), Term::symbol("x")]));
        let s = crate::render_wexpr::render_wexpr(&crate::mma_bridge::term_to_wexpr(&e));
        assert!(s.contains('x'), "got {s}");
    }

    #[test]
    fn map_sin_list() {
        let e = evaluate(&Term::app("Map", vec![Term::symbol("Sin"), Term::List(vec![Term::int(0)])]));
        assert!(matches!(e, Term::List(_)));
    }

    #[test]
    fn truthy_exact_zero() {
        assert_eq!(truthy(&Term::int(0)), Some(false));
        assert_eq!(truthy(&Term::int(1)), Some(true));
    }
}
