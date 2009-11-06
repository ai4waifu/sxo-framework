//! Render session arena [`TermId`] as MATLAB text (no `WExpr`).

use athena::{
    ir::Atom,
    numeric::Number,
    Session,
    types::TermId,
    ir::TermNode,
    runtime::values::arena::application_arguments,
    runtime::values::arena::application_head_name,
    runtime::values::arena::number_from_id,
    runtime::values::arena::symbol_name,
};

use crate::shared::render_number;

/// Render engine IR as MATLAB-ish source.
pub fn render_matlab(session: &Session, id: TermId) -> String {
    match session.arena.get(id) {
        Some(TermNode::Atom(a)) => match a {
            Atom::Number(n) => render_number(n),
            Atom::String(s) => format!("'{s}'"),
            Atom::Symbol(_) => symbol_name(session, id).unwrap_or_else(|| "?".into()),
            Atom::Boolean(true) => "true".into(),
            Atom::Boolean(false) => "false".into(),
            Atom::Null => "[]".into(),
        },
        Some(TermNode::Collection { elements: items, .. }) => {
            let items = items.clone();
            if is_matrix_rows(session, &items) {
                let rows: Vec<String> = items
                    .iter()
                    .map(|row| match session.arena.get(*row) {
                        Some(TermNode::Collection { elements: cols, .. }) => {
                            cols.iter().map(|c| render_matlab(session, *c)).collect::<Vec<_>>().join(", ")
                        }
                        _ => render_matlab(session, *row),
                    })
                    .collect();
                format!("[{}]", rows.join("; "))
            }
            else {
                let inner = items.iter().map(|i| render_matlab(session, *i)).collect::<Vec<_>>().join(", ");
                format!("[{inner}]")
            }
        }
        Some(TermNode::Application { .. }) => {
            let args = application_arguments(session, id).unwrap_or_default();
            if let Some(infix) = try_infix(session, id, &args) {
                return infix;
            }
            let h = match application_head_name(session, id) {
                Some(n) => head_matlab_name(&n),
                None => "?".into(),
            };
            let inner = args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(", ");
            format!("{h}({inner})")
        }
        None => format!("TermId({})", id.0),
    }
}

fn head_matlab_name(name: &str) -> String {
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
    .to_string()
}

fn try_infix(session: &Session, id: TermId, args: &[TermId]) -> Option<String> {
    let name = application_head_name(session, id)?;
    match name.as_str() {
        "Plus" if args.len() >= 2 => Some(args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(" + ")),
        "Times" if args.len() >= 2 => {
            if args.len() == 2 && is_neg_one(session, args[0]) {
                return Some(format!("-{}", render_matlab(session, args[1])));
            }
            Some(args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join("*"))
        }
        "Power" if args.len() == 2 => {
            Some(format!("{}^{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Subtract" if args.len() == 2 => {
            Some(format!("{} - {}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Divide" if args.len() == 2 => {
            Some(format!("{}/{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Mldivide" if args.len() == 2 => {
            Some(format!("{}\\{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "DotTimes" if args.len() == 2 => {
            Some(format!("{}.*{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "DotDivide" if args.len() == 2 => {
            Some(format!("{}./{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "DotPower" if args.len() == 2 => {
            Some(format!("{}.^{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Span" | "Range" if args.len() == 2 => {
            Some(format!("{}:{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Span" | "Range" if args.len() == 3 => Some(format!(
            "{}:{}:{}",
            render_matlab(session, args[0]),
            render_matlab(session, args[1]),
            render_matlab(session, args[2])
        )),
        _ => None,
    }
}

fn is_neg_one(session: &Session, id: TermId) -> bool {
    matches!(number_from_id(session, id), Some(n) if *n == Number::small_int(-1))
}

fn is_matrix_rows(session: &Session, items: &[TermId]) -> bool {
    items.len() > 1 && items.iter().all(|row| matches!(session.arena.get(*row), Some(TermNode::Collection { .. })))
}
