//! MATLAB dialect via oaks **language AST** (`MatlabBuilder`) → session arena [`TermId`].
//!
//! Formal path: oak CST → [`MatlabRoot`] / [`Statement`] / [`Expression`] → arena nodes.
//! Do not expand GreenTree / `MatlabTokenType` leaf walking here.

use oak_core::{Builder, source::SourceText};
use oak_matlab::{
    MatlabBuilder, MatlabLanguage,
    ast::{BinaryExpr, Expression, MatlabRoot, Statement, UnaryExpr},
    lexer::token_type::MatlabTokenType,
};

use athena::{
    ir::Atom,
    Session,
    types::SourceSpan,
    types::TermId,
    ir::TermNode,
    runtime::values::arena::application_arguments,
    runtime::values::arena::application_head_name,
    runtime::values::numeric_clone::clone_number,
    runtime::values::arena::get_kind,
    runtime::values::arena::push_application_named,
    runtime::values::arena::push_bool,
    runtime::values::arena::push_int,
    runtime::values::arena::push_list,
    runtime::values::arena::push_null,
    runtime::values::arena::push_symbol_name,
    runtime::values::arena::symbol_name,
};
use sxo_types::SxoError;

use crate::shared::parse_number_literal;

/// Parse MATLAB text into a session arena [`TermId`] (no evaluate).
pub fn parse_matlab(session: &mut Session, input: &str) -> Result<TermId, SxoError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SxoError::new("matlab: empty input"));
    }

    let language = MatlabLanguage::default();
    let builder = MatlabBuilder::new(&language);
    let source = SourceText::new(trimmed);
    let mut oak_session = oak_core::ParseSession::<MatlabLanguage>::default();
    let output = builder.build(&source, &[], &mut oak_session);
    let root = output.result.map_err(|e| SxoError::new(format!("matlab(oak): {e:?}")))?;
    lower_root(session, &root)
}

fn push_number(session: &mut Session, n: athena::numeric::Number) -> TermId {
    session.arena.push(TermNode::Atom(Atom::Number(clone_number(&n))), SourceSpan::default())
}

fn push_string(session: &mut Session, s: String) -> TermId {
    session.arena.push(TermNode::Atom(Atom::String(s)), SourceSpan::default())
}

fn lower_root(session: &mut Session, root: &MatlabRoot) -> Result<TermId, SxoError> {
    let mut items = Vec::with_capacity(root.items.len());
    for stmt in &root.items {
        items.push(lower_stmt(session, stmt)?);
    }
    match items.len() {
        0 => Err(SxoError::new("matlab(oak): empty root")),
        1 => Ok(items.remove(0)),
        _ => Ok(push_application_named(session, "CompoundExpression", items)),
    }
}

fn lower_stmt(session: &mut Session, stmt: &Statement) -> Result<TermId, SxoError> {
    match stmt {
        Statement::Expr(expr) => lower_expr(session, expr),
        Statement::If { condition, then_body, elseifs, else_body, .. } => {
            let mut else_term = compound_stmts(session, else_body)?;
            for (cond, body) in elseifs.iter().rev() {
                let then_t = compound_stmts(session, body)?;
                let cond_t = lower_expr(session, cond)?;
                else_term = push_application_named(session, "If", vec![cond_t, then_t, else_term]);
            }
            let then_t = compound_stmts(session, then_body)?;
            let cond_t = lower_expr(session, condition)?;
            let else_is_null = matches!(get_kind(session, else_term), Some(TermNode::Atom(Atom::Null)));
            if else_is_null && elseifs.is_empty() {
                Ok(push_application_named(session, "If", vec![cond_t, then_t]))
            }
            else {
                Ok(push_application_named(session, "If", vec![cond_t, then_t, else_term]))
            }
        }
        Statement::While { condition, body, .. } => {
            let cond_t = lower_expr(session, condition)?;
            let body_t = compound_stmts(session, body)?;
            Ok(push_application_named(session, "While", vec![cond_t, body_t]))
        }
        Statement::For { header, body, .. } => {
            let header_t = lower_expr(session, header)?;
            let body_t = compound_stmts(session, body)?;
            if application_head_name(session, header_t).as_deref() == Some("Set") {
                if let Some(args) = application_arguments(session, header_t) {
                    if args.len() == 2 {
                        return Ok(push_application_named(session, "For", vec![args[0], args[1], body_t]));
                    }
                }
            }
            let underscore = push_symbol_name(session, "_");
            Ok(push_application_named(session, "For", vec![underscore, header_t, body_t]))
        }
        Statement::Try { body, catch_body, .. } => {
            let body_t = compound_stmts(session, body)?;
            let catch_t = compound_stmts(session, catch_body)?;
            Ok(push_application_named(session, "Try", vec![body_t, catch_t]))
        }
        Statement::Error { .. } => Err(SxoError::new("matlab(oak): error node")),
    }
}

fn compound_stmts(session: &mut Session, stmts: &[Statement]) -> Result<TermId, SxoError> {
    let mut items = Vec::with_capacity(stmts.len());
    for s in stmts {
        items.push(lower_stmt(session, s)?);
    }
    Ok(compound_or_single(session, items))
}

fn compound_or_single(session: &mut Session, mut items: Vec<TermId>) -> TermId {
    match items.len() {
        0 => push_null(session),
        1 => items.remove(0),
        _ => push_application_named(session, "CompoundExpression", items),
    }
}

fn lower_expr(session: &mut Session, expr: &Expression) -> Result<TermId, SxoError> {
    match expr {
        Expression::Symbol(id) => {
            if id.name == "end" {
                Ok(push_symbol_name(session, "end"))
            }
            else if id.name == "true" {
                Ok(push_bool(session, true))
            }
            else if id.name == "false" {
                Ok(push_bool(session, false))
            }
            else {
                Ok(push_symbol_name(session, &id.name))
            }
        }
        Expression::Literal { value, .. } => {
            let text = value.trim();
            if let Some(n) = parse_number_literal(text) {
                return Ok(push_number(session, n));
            }
            if (text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
            {
                Ok(push_string(session, text[1..text.len() - 1].to_string()))
            }
            else {
                Ok(push_symbol_name(session, text))
            }
        }
        Expression::Array { rows, .. } => {
            if rows.len() == 1 {
                let mut items = Vec::with_capacity(rows[0].len());
                for cell in &rows[0] {
                    items.push(lower_expr(session, cell)?);
                }
                Ok(push_list(session, items))
            }
            else {
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    let mut cols = Vec::with_capacity(row.len());
                    for cell in row {
                        cols.push(lower_expr(session, cell)?);
                    }
                    out.push(push_list(session, cols));
                }
                Ok(push_list(session, out))
            }
        }
        Expression::Call { head, arguments, .. } => {
            let mut expr_t = lower_expr(session, head)?;
            if let Some(name) = symbol_name(session, expr_t) {
                expr_t = push_symbol_name(session, &map_matlab_head(&name));
            }
            let mut args = Vec::with_capacity(arguments.len());
            for a in arguments {
                args.push(lower_expr(session, a)?);
            }
            let is_part_base = matches!(get_kind(session, expr_t), Some(TermNode::Collection { .. }))
                || application_head_name(session, expr_t).as_deref() == Some("Part");
            if is_part_base {
                let mut part_args = vec![expr_t];
                part_args.extend(args);
                Ok(push_application_named(session, "Part", part_args))
            }
            else if let Some(head_name) = symbol_name(session, expr_t) {
                Ok(push_application_named(session, &head_name, args))
            }
            else {
                // Non-symbol head → `Application[head, args…]` for Athena `EvalDynamic`.
                let mut wrapped = vec![expr_t];
                wrapped.extend(args);
                Ok(push_application_named(session, "Application", wrapped))
            }
        }
        Expression::Binary(bin) => lower_binary(session, bin),
        Expression::Prefix(u) => lower_prefix(session, u),
        Expression::Postfix(u) => lower_postfix(session, u),
        Expression::Grouped { expression, .. } => lower_expr(session, expression),
    }
}

fn lower_binary(session: &mut Session, bin: &BinaryExpr) -> Result<TermId, SxoError> {
    let l = lower_expr(session, &bin.lhs)?;
    let r = lower_expr(session, &bin.rhs)?;
    Ok(match bin.operator {
        MatlabTokenType::Plus => push_application_named(session, "Plus", vec![l, r]),
        MatlabTokenType::Minus => push_application_named(session, "Subtract", vec![l, r]),
        MatlabTokenType::Times => push_application_named(session, "Times", vec![l, r]),
        MatlabTokenType::DotTimes => push_application_named(session, "DotTimes", vec![l, r]),
        MatlabTokenType::Divide => push_application_named(session, "Divide", vec![l, r]),
        MatlabTokenType::DotDivide => push_application_named(session, "DotDivide", vec![l, r]),
        MatlabTokenType::LeftDivide => push_application_named(session, "LinearSolve", vec![l, r]),
        MatlabTokenType::DotLeftDivide => push_application_named(session, "DotLeftDivide", vec![l, r]),
        MatlabTokenType::Power => push_application_named(session, "Power", vec![l, r]),
        MatlabTokenType::DotPower => push_application_named(session, "DotPower", vec![l, r]),
        MatlabTokenType::Assign => push_application_named(session, "Set", vec![l, r]),
        MatlabTokenType::Equal => push_application_named(session, "Equal", vec![l, r]),
        MatlabTokenType::NotEqual => push_application_named(session, "Unequal", vec![l, r]),
        MatlabTokenType::Less => push_application_named(session, "Less", vec![l, r]),
        MatlabTokenType::Greater => push_application_named(session, "Greater", vec![l, r]),
        MatlabTokenType::LessEqual => push_application_named(session, "LessEqual", vec![l, r]),
        MatlabTokenType::GreaterEqual => push_application_named(session, "GreaterEqual", vec![l, r]),
        MatlabTokenType::AndAnd | MatlabTokenType::And => push_application_named(session, "And", vec![l, r]),
        MatlabTokenType::OrOr | MatlabTokenType::Or => push_application_named(session, "Or", vec![l, r]),
        MatlabTokenType::Colon => flatten_range(session, l, r),
        other => {
            return Err(SxoError::new(format!("matlab(ast): unsupported binary {other:?}")));
        }
    })
}

fn flatten_range(session: &mut Session, left: TermId, right: TermId) -> TermId {
    if application_head_name(session, left).as_deref() == Some("Range") {
        if let Some(args) = application_arguments(session, left) {
            if args.len() == 2 {
                // MATLAB `start:step:end` → Athena `Range[start, end, step]`.
                return push_application_named(session, "Range", vec![args[0], right, args[1]]);
            }
        }
    }
    push_application_named(session, "Range", vec![left, right])
}

fn lower_prefix(session: &mut Session, u: &UnaryExpr) -> Result<TermId, SxoError> {
    let e = lower_expr(session, &u.operand)?;
    Ok(match u.operator {
        MatlabTokenType::Minus => {
            let neg1 = push_int(session, -1);
            push_application_named(session, "Times", vec![neg1, e])
        }
        MatlabTokenType::Plus => e,
        MatlabTokenType::Not => push_application_named(session, "Not", vec![e]),
        other => return Err(SxoError::new(format!("matlab(ast): unsupported prefix {other:?}"))),
    })
}

fn lower_postfix(session: &mut Session, u: &UnaryExpr) -> Result<TermId, SxoError> {
    let e = lower_expr(session, &u.operand)?;
    Ok(match u.operator {
        MatlabTokenType::Transpose | MatlabTokenType::DotTranspose => push_application_named(session, "Transpose", vec![e]),
        other => return Err(SxoError::new(format!("matlab(ast): unsupported postfix {other:?}"))),
    })
}

/// Map common MATLAB function names to engine heads used by evaluate/render.
fn map_matlab_head(name: &str) -> String {
    match name {
        "sin" => "Sin".to_string(),
        "cos" => "Cos".to_string(),
        "tan" => "Tan".to_string(),
        "exp" => "Exp".to_string(),
        "log" => "Log".to_string(),
        "diff" => "D".to_string(),
        "simplify" => "Simplify".to_string(),
        "int" => "Integrate".to_string(),
        "sqrt" => "Sqrt".to_string(),
        "abs" => "Abs".to_string(),
        "factorial" => "Factorial".to_string(),
        "zeros" => "Zeros".to_string(),
        "ones" => "Ones".to_string(),
        "eye" => "Eye".to_string(),
        "size" => "Size".to_string(),
        "length" => "Length".to_string(),
        "det" => "Det".to_string(),
        "sum" => "Sum".to_string(),
        "linsolve" => "LinearSolve".to_string(),
        other => other.to_string(),
    }
}
