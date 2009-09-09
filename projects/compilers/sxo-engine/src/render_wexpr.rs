//! Render [`WExpr`] as Wolfram text.

use crate::number_literal::render_number;
use crate::term::Atom;
use crate::wexpr::{WAtom, WExpr};

/// Render a Wolfram expression.
pub fn render_wexpr(expr: &WExpr) -> String {
    match expr {
        WExpr::Atom(a) => match a {
            WAtom::Number(n) => render_number(n),
            WAtom::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            WAtom::Symbol(s) => s.clone(),
        },
        WExpr::List(items) => {
            let inner = items.iter().map(render_wexpr).collect::<Vec<_>>().join(", ");
            format!("{{{inner}}}")
        }
        WExpr::Call { head, args } => {
            if let Some(infix) = try_infix(head, args) {
                return infix;
            }
            let h = render_wexpr(head);
            let inner = args.iter().map(render_wexpr).collect::<Vec<_>>().join(", ");
            format!("{h}[{inner}]")
        }
    }
}

fn try_infix(head: &WExpr, args: &[WExpr]) -> Option<String> {
    let name = match head {
        WExpr::Atom(WAtom::Symbol(s)) => s.as_str(),
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
            Some(format!("{}^{}", maybe_paren(&args[0], Prec::Pow), maybe_paren(&args[1], Prec::Pow)))
        }
        "Subtract" if args.len() == 2 => {
            Some(format!("{} - {}", maybe_paren(&args[0], Prec::Add), maybe_paren(&args[1], Prec::Mul)))
        }
        "Divide" if args.len() == 2 => {
            Some(format!("{}/{}", maybe_paren(&args[0], Prec::Mul), maybe_paren(&args[1], Prec::Pow)))
        }
        "Rule" if args.len() == 2 => Some(format!("{} -> {}", render_wexpr(&args[0]), render_wexpr(&args[1]))),
        "RuleDelayed" if args.len() == 2 => Some(format!("{} :> {}", render_wexpr(&args[0]), render_wexpr(&args[1]))),
        "ReplaceAll" if args.len() == 2 => {
            Some(format!("{} /. {}", maybe_paren(&args[0], Prec::Replace), maybe_paren(&args[1], Prec::Replace)))
        }
        "Function" if args.len() == 1 => Some(format!("{} &", maybe_paren(&args[0], Prec::Function))),
        "Part" if args.len() == 2 => Some(format!("{}[[{}]]", maybe_paren(&args[0], Prec::Part), render_wexpr(&args[1]))),
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

fn prec(expr: &WExpr) -> Prec {
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

fn maybe_paren(expr: &WExpr, parent: Prec) -> String {
    let s = render_wexpr(expr);
    if prec(expr) < parent { format!("({s})") } else { s }
}

#[allow(dead_code)]
fn watom_to_atom(a: &WAtom) -> Atom {
    match a {
        WAtom::Number(n) => Atom::Number(n.clone()),
        WAtom::String(s) => Atom::String(s.clone()),
        WAtom::Symbol(s) => Atom::Symbol(s.clone()),
    }
}
