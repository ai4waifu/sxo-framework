//! Render engine [`Term`] as MATLAB text (no `WExpr`).

use athena::{Atom, Term};

use crate::shared::render_number;

/// Render engine IR as MATLAB-ish source.
pub fn render_matlab(expr: &Term) -> String {
    match expr {
        Term::Atom(a) => match a {
            Atom::Number(n) => render_number(n),
            Atom::String(s) => format!("'{s}'"),
            Atom::Symbol(s) => s.clone(),
            Atom::Boolean(true) => "true".into(),
            Atom::Boolean(false) => "false".into(),
            Atom::Null => "[]".into(),
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
        Term::Application { head, arguments: args } => {
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
            "Zeros" => "zeros",
            "Ones" => "ones",
            "Eye" | "IdentityMatrix" => "eye",
            "Size" | "Dimensions" => "size",
            "Length" => "length",
            "Det" => "det",
            "Sum" => "sum",
            "LinearSolve" => "linsolve",
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
        "Mldivide" if args.len() == 2 => Some(format!("{}\\{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "DotTimes" if args.len() == 2 => Some(format!("{}.*{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "DotDivide" if args.len() == 2 => Some(format!("{}./{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "DotPower" if args.len() == 2 => Some(format!("{}.^{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "Span" if args.len() == 2 => Some(format!("{}:{}", render_matlab(&args[0]), render_matlab(&args[1]))),
        "Span" if args.len() == 3 => {
            Some(format!("{}:{}:{}", render_matlab(&args[0]), render_matlab(&args[1]), render_matlab(&args[2])))
        },
        _ => None,
    }
}

fn is_matrix_rows(items: &[Term]) -> bool {
    items.len() > 1 && items.iter().all(|row| matches!(row, Term::List(_)))
}
