//! Render Mathematica Form as Wolfram text.

use crate::{
    form::{WolframAtom, WolframForm},
    number_literal::render_number,
};

/// Render a Wolfram expression.
pub fn render(expr: &WolframForm) -> String {
    match expr {
        WolframForm::Atom(a) => match a {
            WolframAtom::Number(n) => render_number(n),
            WolframAtom::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            WolframAtom::Symbol(s) => s.clone(),
        },
        WolframForm::List(items) => {
            let inner = items.iter().map(render).collect::<Vec<_>>().join(", ");
            format!("{{{inner}}}")
        }
        WolframForm::Call { head, args } => {
            if let Some(infix) = try_infix(head, args) {
                return infix;
            }
            let h = render(head);
            let inner = args.iter().map(render).collect::<Vec<_>>().join(", ");
            format!("{h}[{inner}]")
        }
    }
}

fn try_infix(head: &WolframForm, args: &[WolframForm]) -> Option<String> {
    let name = match head {
        WolframForm::Atom(WolframAtom::Symbol(s)) => s.as_str(),
        _ => return None,
    };
    match name {
        "Plus" if args.len() >= 2 => Some(args.iter().map(|a| maybe_paren(a, Prec::Add)).collect::<Vec<_>>().join(" + ")),
        "Times" if args.len() >= 2 => {
            if args.len() == 2 && args[0].is_neg_one() {
                return Some(format!("-{}", maybe_paren(&args[1], Prec::Unary)));
            }
            Some(args.iter().map(|a| maybe_paren(a, Prec::Mul)).collect::<Vec<_>>().join("*"))
        }
        "Power" if args.len() == 2 => {
            // Negative / rational number atoms print with leading `-` or `/`.
            // Without parens, Wolfram re-parses `(-8)^(1/3)` as `-8^1/3`.
            Some(format!("{}^{}", power_operand(&args[0]), power_operand(&args[1])))
        }
        "Subtract" if args.len() == 2 => {
            Some(format!("{} - {}", maybe_paren(&args[0], Prec::Add), maybe_paren(&args[1], Prec::Mul)))
        }
        "Divide" if args.len() == 2 => {
            Some(format!("{}/{}", maybe_paren(&args[0], Prec::Mul), maybe_paren(&args[1], Prec::Pow)))
        }
        "Rule" if args.len() == 2 => Some(format!("{} -> {}", render(&args[0]), render(&args[1]))),
        "RuleDelayed" if args.len() == 2 => Some(format!("{} :> {}", render(&args[0]), render(&args[1]))),
        "ReplaceAll" if args.len() == 2 => {
            Some(format!("{} /. {}", maybe_paren(&args[0], Prec::Replace), maybe_paren(&args[1], Prec::Replace)))
        }
        "Function" if args.len() == 1 => Some(format!("{} &", maybe_paren(&args[0], Prec::Function))),
        "Part" if args.len() == 2 => Some(format!("{}[[{}]]", maybe_paren(&args[0], Prec::Part), render(&args[1]))),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Replace = 1,
    Function = 2,
    Add = 3,
    Mul = 4,
    Pow = 5,
    Unary = 6,
    Part = 7,
    Atom = 8,
}

fn prec(expr: &WolframForm) -> Prec {
    match expr.head_name() {
        Some("ReplaceAll") | Some("ReplaceRepeated") => Prec::Replace,
        Some("Function") => Prec::Function,
        Some("Plus") | Some("Subtract") => Prec::Add,
        Some("Times") | Some("Divide") => Prec::Mul,
        Some("Power") => Prec::Pow,
        Some("Part") => Prec::Part,
        _ => Prec::Atom,
    }
}

fn maybe_paren(expr: &WolframForm, parent: Prec) -> String {
    let s = render(expr);
    if prec(expr) < parent { format!("({s})") } else { s }
}

fn power_operand(expr: &WolframForm) -> String {
    let s = render(expr);
    if power_atom_needs_paren(expr) || prec(expr) < Prec::Pow { format!("({s})") } else { s }
}

fn power_atom_needs_paren(expr: &WolframForm) -> bool {
    match expr {
        WolframForm::Atom(WolframAtom::Number(n)) => {
            let text = n.to_render_string();
            text.starts_with('-') || text.contains('/')
        }
        _ => false,
    }
}
