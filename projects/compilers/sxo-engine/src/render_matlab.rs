//! Render engine [`Term`] as MATLAB text (no `WExpr`).

use crate::number_literal::render_number;
use crate::term::{Atom, Term};

/// Render engine IR as MATLAB-ish source.
pub fn render_matlab(expr: &Term) -> String {
    match expr {
        Term::Atom(a) => match a {
            Atom::Number(n) => render_number(n),
            Atom::String(s) => format!("'{s}'"),
            Atom::Symbol(s) => s.clone(),
        },
        Term::List(items) => {
            if is_matrix_rows(items) {
                let rows: Vec<String> = items
                    .iter()
                    .map(|row| match row {
                        Term::List(cols) => cols.iter().map(render_matlab).collect::<Vec<_>>().join(", "),
                        other => render_matlab(other),
                    })
                    .collect();
                format!("[{}]", rows.join("; "))
            }
            else {
                let inner = items.iter().map(render_matlab).collect::<Vec<_>>().join(", ");
                format!("[{inner}]")
            }
        }
        Term::App { head, args } => {
            if let Some(infix) = try_infix(head, args) {
                return infix;
            }
            let h = match head_matlab_name(head) {
                Some(n) => n,
                None => render_matlab(head),
            };
            let inner = args.iter().map(render_matlab).collect::<Vec<_>>().join(", ");
            format!("{h}({inner})")
        }
    }
}

fn head_matlab_name(head: &Term) -> Option<String> {
    let name = head.head_name()?;
    Some(
        match name {
            "Sin" => "sin",
            "Cos" => "cos",
            "Tan" => "tan",
            "Exp" => "exp",
            "Log" => "log",
            "D" => "diff",
            "Simplify" => "simplify",
            "Integrate" => "int",
            "Sqrt" => "sqrt",
            "Abs" => "abs",
            "Factorial" => "factorial",
            other => other,
        }
        .to_string(),
    )
}

fn try_infix(head: &Term, args: &[Term]) -> Option<String> {
    let name = head.head_name()?;
    match name {
        "Plus" if args.len() >= 2 => Some(args.iter().map(render_matlab).collect::<Vec<_>>().join(" + ")),
        "Times" if args.len() >= 2 => {
            if args.len() == 2 && args[0].is_neg_one() {
                return Some(format!("-{}", render_matlab(&args[1])));
            }
            Some(args.iter().map(render_matlab).collect::<Vec<_>>().join("*"))
        }
        "Power" if args.len() == 2 => Some(format!("{}^{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "Subtract" if args.len() == 2 => Some(format!("{} - {}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "Divide" if args.len() == 2 => Some(format!("{}/{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        _ => None,
    }
}

fn is_matrix_rows(items: &[Term]) -> bool {
    items.len() > 1 && items.iter().all(|row| matches!(row, Term::List(_)))
}
