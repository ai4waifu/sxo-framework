//! Render MATLAB Form or session arena [`TermId`] as MATLAB text (no `WExpr`).

use athena::{
    Session,
    ir::{Atom, MathematicalConstant, TermNode},
    numeric::Number,
    runtime::values::arena::{application_arguments, number_from_id, symbol_name},
    types::TermId,
};

use crate::{
    form::{MatlabAtom, MatlabForm},
    number_literal::render_number,
    surface::application_surface_name,
};

/// Render a retained [`MatlabForm`] without materializing arena terms.
pub fn render_matlab_form(form: &MatlabForm) -> String {
    match form {
        MatlabForm::Atom(a) => match a {
            MatlabAtom::Number(n) => render_number(n),
            MatlabAtom::String(s) => format!("'{s}'"),
            MatlabAtom::Symbol(s) => s.clone(),
            MatlabAtom::Bool(true) => "true".into(),
            MatlabAtom::Bool(false) => "false".into(),
            MatlabAtom::Null => "[]".into(),
        },
        MatlabForm::List(items) => {
            if is_form_matrix_rows(items) {
                let rows: Vec<String> = items
                    .iter()
                    .map(|row| match row {
                        MatlabForm::List(cols) => cols.iter().map(render_matlab_form).collect::<Vec<_>>().join(", "),
                        other => render_matlab_form(other),
                    })
                    .collect();
                format!("[{}]", rows.join("; "))
            }
            else {
                let inner = items.iter().map(render_matlab_form).collect::<Vec<_>>().join(", ");
                format!("[{inner}]")
            }
        }
        MatlabForm::Call { head, args } => {
            if let Some(infix) = try_form_infix(head, args) {
                return infix;
            }
            if head == "Cell" {
                return render_cell_form(args);
            }
            if head == "Command" {
                return render_command_form(args);
            }
            if head == "Global" || head == "Persistent" {
                let kw = if head == "Global" { "global" } else { "persistent" };
                if args.is_empty() {
                    return kw.into();
                }
                let names = args.iter().map(render_matlab_form).collect::<Vec<_>>().join(" ");
                return format!("{kw} {names}");
            }
            if head == "Member" && args.len() == 2 {
                return format!("{}.{}", render_matlab_form(&args[0]), render_matlab_form(&args[1]));
            }
            if head == "Parfor" && args.len() == 3 {
                return format!(
                    "parfor {}={}, {}, end",
                    render_matlab_form(&args[0]),
                    render_matlab_form(&args[1]),
                    render_matlab_form(&args[2])
                );
            }
            if head == "Spmd" && args.len() == 1 {
                return format!("spmd, {}, end", render_matlab_form(&args[0]));
            }
            // `Application[head, args…]` / ApplyHead residual → `head(args…)`.
            if head == "Application" && !args.is_empty() {
                let callee = render_matlab_form(&args[0]);
                let inner = args[1..].iter().map(render_matlab_form).collect::<Vec<_>>().join(", ");
                return format!("{callee}({inner})");
            }
            let h = head_matlab_name(head);
            let inner = args.iter().map(render_matlab_form).collect::<Vec<_>>().join(", ");
            format!("{h}({inner})")
        }
    }
}

/// Render engine IR as MATLAB-ish source.
pub fn render_matlab(session: &Session, id: TermId) -> String {
    match session.arena.get(id) {
        Some(TermNode::Atom(a)) => match a {
            Atom::Number(n) => render_number(n),
            Atom::String(s) => format!("'{s}'"),
            Atom::Symbol(_) => match symbol_name(session, id).as_deref() {
                Some("Infinity") => "Inf".into(),
                Some(other) => other.into(),
                None => "?".into(),
            },
            Atom::Boolean(true) => "true".into(),
            Atom::Boolean(false) => "false".into(),
            Atom::Null => "[]".into(),
            Atom::Constant(MathematicalConstant::Infinity) => "Inf".into(),
            Atom::Constant(c) => c.debug_label().into(),
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
            if application_surface_name(session, id).as_deref() == Some("Indeterminate") {
                return "NaN".into();
            }
            let args = application_arguments(session, id).unwrap_or_default();
            if let Some(infix) = try_infix(session, id, &args) {
                return infix;
            }
            let h = match application_surface_name(session, id) {
                Some(n) if n == "Cell" => {
                    return render_cell_term(session, &args);
                }
                Some(n) if n == "Command" => {
                    return render_command_term(session, &args);
                }
                Some(n) if n == "Global" || n == "Persistent" => {
                    let kw = if n == "Global" { "global" } else { "persistent" };
                    if args.is_empty() {
                        return kw.into();
                    }
                    let names = args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(" ");
                    return format!("{kw} {names}");
                }
                Some(n) if n == "Member" && args.len() == 2 => {
                    return format!("{}.{}", render_matlab(session, args[0]), render_matlab(session, args[1]));
                }
                Some(n) if n == "Parfor" && args.len() == 3 => {
                    return format!(
                        "parfor {}={}, {}, end",
                        render_matlab(session, args[0]),
                        render_matlab(session, args[1]),
                        render_matlab(session, args[2])
                    );
                }
                Some(n) if n == "Spmd" && args.len() == 1 => {
                    return format!("spmd, {}, end", render_matlab(session, args[0]));
                }
                Some(n) if n == "Application" && !args.is_empty() => {
                    let callee = render_matlab(session, args[0]);
                    let inner = args[1..].iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(", ");
                    return format!("{callee}({inner})");
                }
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
        "Det" | "Determinant" => "det",
        "Sum" => "sum",
        "LinearSolve" => "linsolve",
        "Minus" => "-",
        "FunctionHandle" => "@",
        "Indeterminate" => "NaN",
        "Infinity" => "Inf",
        other => other,
    }
    .to_string()
}

fn render_command_form(args: &[MatlabForm]) -> String {
    args.iter().map(render_matlab_form).collect::<Vec<_>>().join(" ")
}

fn render_command_term(session: &Session, args: &[TermId]) -> String {
    args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(" ")
}

fn render_cell_form(args: &[MatlabForm]) -> String {
    match args {
        [MatlabForm::List(items)] => {
            if is_form_matrix_rows(items) {
                let rows: Vec<String> = items
                    .iter()
                    .map(|row| match row {
                        MatlabForm::List(cols) => cols.iter().map(render_matlab_form).collect::<Vec<_>>().join(", "),
                        other => render_matlab_form(other),
                    })
                    .collect();
                format!("{{{}}}", rows.join("; "))
            }
            else {
                let inner = items.iter().map(render_matlab_form).collect::<Vec<_>>().join(", ");
                format!("{{{inner}}}")
            }
        }
        _ => {
            let inner = args.iter().map(render_matlab_form).collect::<Vec<_>>().join(", ");
            format!("{{{inner}}}")
        }
    }
}

fn render_cell_term(session: &Session, args: &[TermId]) -> String {
    match args {
        [only] => match session.arena.get(*only) {
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
                    format!("{{{}}}", rows.join("; "))
                }
                else {
                    let inner = items.iter().map(|i| render_matlab(session, *i)).collect::<Vec<_>>().join(", ");
                    format!("{{{inner}}}")
                }
            }
            _ => format!("{{{}}}", render_matlab(session, *only)),
        },
        _ => {
            let inner = args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(", ");
            format!("{{{inner}}}")
        }
    }
}

fn try_form_infix(head: &str, args: &[MatlabForm]) -> Option<String> {
    match head {
        "Plus" if args.len() >= 2 => Some(args.iter().map(render_matlab_form).collect::<Vec<_>>().join(" + ")),
        "Times" if args.len() >= 2 => {
            if args.len() == 2 && is_form_neg_one(&args[0]) {
                return Some(format!("-{}", render_matlab_form(&args[1])));
            }
            Some(args.iter().map(render_matlab_form).collect::<Vec<_>>().join("*"))
        }
        "Minus" if args.len() == 1 => Some(format!("-{}", render_matlab_form(&args[0]))),
        "Power" if args.len() == 2 => Some(format!("{}^{}", form_power_operand(&args[0]), form_power_operand(&args[1]))),
        "Subtract" if args.len() == 2 => Some(format!("{} - {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Divide" if args.len() == 2 => Some(format!("{}/{}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "LinearSolve" | "Mldivide" if args.len() == 2 => {
            Some(format!("{}\\{}", render_matlab_form(&args[0]), render_matlab_form(&args[1])))
        }
        "DotTimes" if args.len() == 2 => Some(format!("{}.*{}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "DotDivide" if args.len() == 2 => Some(format!("{}./{}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "DotPower" if args.len() == 2 => Some(format!("{}.^{}", form_power_operand(&args[0]), form_power_operand(&args[1]))),
        "ElementwiseAnd" if args.len() == 2 => {
            Some(format!("{} & {}", render_matlab_form(&args[0]), render_matlab_form(&args[1])))
        }
        "ElementwiseOr" if args.len() == 2 => {
            Some(format!("{} | {}", render_matlab_form(&args[0]), render_matlab_form(&args[1])))
        }
        "And" if args.len() == 2 => Some(format!("{} && {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Or" if args.len() == 2 => Some(format!("{} || {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Equal" if args.len() == 2 => Some(format!("{} == {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Unequal" if args.len() == 2 => Some(format!("{} ~= {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Less" if args.len() == 2 => Some(format!("{} < {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Greater" if args.len() == 2 => Some(format!("{} > {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "LessEqual" if args.len() == 2 => Some(format!("{} <= {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "GreaterEqual" if args.len() == 2 => {
            Some(format!("{} >= {}", render_matlab_form(&args[0]), render_matlab_form(&args[1])))
        }
        "Transpose" if args.len() == 1 => Some(format!("{}.'", render_matlab_form(&args[0]))),
        "ConjugateTranspose" if args.len() == 1 => Some(format!("{}'", render_matlab_form(&args[0]))),
        "Span" | "Range" if args.len() == 2 => {
            Some(format!("{}:{}", render_matlab_form(&args[0]), render_matlab_form(&args[1])))
        }
        "Span" | "Range" if args.len() == 3 => {
            Some(format!("{}:{}:{}", render_matlab_form(&args[0]), render_matlab_form(&args[1]), render_matlab_form(&args[2])))
        }
        "Part" if args.len() >= 2 => {
            let base = render_matlab_form(&args[0]);
            let idxs = args[1..].iter().map(render_matlab_form).collect::<Vec<_>>().join(", ");
            Some(format!("{base}({idxs})"))
        }
        "Set" if args.len() == 2 => Some(format!("{} = {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "Function" if args.len() == 2 => Some(format!("@({}) {}", render_matlab_form(&args[0]), render_matlab_form(&args[1]))),
        "FunctionHandle" if args.len() == 1 => Some(format!("@{}", render_matlab_form(&args[0]))),
        "CompoundExpression" if !args.is_empty() => Some(args.iter().map(render_matlab_form).collect::<Vec<_>>().join("; ")),
        _ => None,
    }
}

fn is_form_neg_one(form: &MatlabForm) -> bool {
    matches!(form, MatlabForm::Atom(MatlabAtom::Number(n)) if *n == Number::small_int(-1))
}

fn form_power_operand(form: &MatlabForm) -> String {
    let s = render_matlab_form(form);
    if form_number_needs_power_paren(form) || form_compound_needs_power_paren(form) { format!("({s})") } else { s }
}

fn form_number_needs_power_paren(form: &MatlabForm) -> bool {
    match form {
        MatlabForm::Atom(MatlabAtom::Number(n)) => {
            let text = n.to_render_string();
            text.starts_with('-') || text.contains('/')
        }
        _ => false,
    }
}

fn form_compound_needs_power_paren(form: &MatlabForm) -> bool {
    matches!(form.head_name(), Some("Plus" | "Subtract" | "Times" | "Divide" | "Power" | "DotPower" | "Minus"))
}

fn is_form_matrix_rows(items: &[MatlabForm]) -> bool {
    items.len() > 1 && items.iter().all(|row| matches!(row, MatlabForm::List(_)))
}

fn try_infix(session: &Session, id: TermId, args: &[TermId]) -> Option<String> {
    let name = application_surface_name(session, id)?;
    match name.as_str() {
        "Plus" if args.len() >= 2 => Some(args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(" + ")),
        "Times" if args.len() >= 2 => {
            if args.len() == 2 && is_neg_one(session, args[0]) {
                return Some(format!("-{}", render_matlab(session, args[1])));
            }
            Some(args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join("*"))
        }
        "Minus" if args.len() == 1 => Some(format!("-{}", render_matlab(session, args[0]))),
        "Power" if args.len() == 2 => Some(format!("{}^{}", power_operand(session, args[0]), power_operand(session, args[1]))),
        "Subtract" if args.len() == 2 => {
            Some(format!("{} - {}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Divide" if args.len() == 2 => Some(format!("{}/{}", render_matlab(session, args[0]), render_matlab(session, args[1]))),
        "LinearSolve" | "Mldivide" if args.len() == 2 => {
            Some(format!("{}\\{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "DotTimes" if args.len() == 2 => {
            Some(format!("{}.*{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "DotDivide" if args.len() == 2 => {
            Some(format!("{}./{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "DotPower" if args.len() == 2 => {
            Some(format!("{}.^{}", power_operand(session, args[0]), power_operand(session, args[1])))
        }
        "ElementwiseAnd" if args.len() == 2 => {
            Some(format!("{} & {}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "ElementwiseOr" if args.len() == 2 => {
            Some(format!("{} | {}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "And" if args.len() == 2 => Some(format!("{} && {}", render_matlab(session, args[0]), render_matlab(session, args[1]))),
        "Or" if args.len() == 2 => Some(format!("{} || {}", render_matlab(session, args[0]), render_matlab(session, args[1]))),
        "Transpose" if args.len() == 1 => Some(format!("{}.'", render_matlab(session, args[0]))),
        "ConjugateTranspose" if args.len() == 1 => Some(format!("{}'", render_matlab(session, args[0]))),
        "Span" | "Range" if args.len() == 2 => {
            Some(format!("{}:{}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Span" | "Range" if args.len() == 3 => Some(format!(
            "{}:{}:{}",
            render_matlab(session, args[0]),
            render_matlab(session, args[1]),
            render_matlab(session, args[2])
        )),
        "Part" if args.len() >= 2 => {
            let base = render_matlab(session, args[0]);
            let idxs = args[1..].iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join(", ");
            Some(format!("{base}({idxs})"))
        }
        "FunctionHandle" if args.len() == 1 => Some(format!("@{}", render_matlab(session, args[0]))),
        "Function" if args.len() == 2 => {
            Some(format!("@({}) {}", render_matlab(session, args[0]), render_matlab(session, args[1])))
        }
        "Set" if args.len() == 2 => Some(format!("{} = {}", render_matlab(session, args[0]), render_matlab(session, args[1]))),
        "CompoundExpression" if !args.is_empty() => {
            Some(args.iter().map(|a| render_matlab(session, *a)).collect::<Vec<_>>().join("; "))
        }
        _ => None,
    }
}

fn is_neg_one(session: &Session, id: TermId) -> bool {
    matches!(number_from_id(session, id), Some(n) if *n == Number::small_int(-1))
}

fn power_operand(session: &Session, id: TermId) -> String {
    let s = render_matlab(session, id);
    if number_needs_power_paren(session, id) || compound_needs_power_paren(session, id) { format!("({s})") } else { s }
}

fn number_needs_power_paren(session: &Session, id: TermId) -> bool {
    match number_from_id(session, id) {
        Some(n) => {
            let text = n.to_render_string();
            text.starts_with('-') || text.contains('/')
        }
        None => false,
    }
}

fn compound_needs_power_paren(session: &Session, id: TermId) -> bool {
    match application_surface_name(session, id).as_deref() {
        Some("Plus" | "Subtract" | "Times" | "Divide" | "Power" | "DotPower" | "Minus") => true,
        _ => false,
    }
}

fn is_matrix_rows(session: &Session, items: &[TermId]) -> bool {
    items.len() > 1 && items.iter().all(|row| matches!(session.arena.get(*row), Some(TermNode::Collection { .. })))
}
