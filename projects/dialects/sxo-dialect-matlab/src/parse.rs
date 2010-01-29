//! MATLAB dialect via oaks **language AST** (`MatlabBuilder`) → [`MatlabForm`].
//!
//! Formal path: oak CST → [`MatlabRoot`] / [`Statement`] / [`Expression`] → [`MatlabForm`].
//! Session [`TermId`] materialization is [`crate::form_to_term`] (transitional).
//! Do not expand GreenTree / `MatlabTokenType` leaf walking here.

use oak_core::{Builder, source::SourceText};
use oak_matlab::{
    MatlabBuilder, MatlabLanguage,
    ast::{BinaryExpr, Expression, MatlabRoot, Statement, UnaryExpr},
    lexer::token_type::MatlabTokenType,
};

use athena::{Session, types::TermId};
use sxo_types::SxoError;

use crate::form::{MatlabAtom, MatlabForm};
use crate::lower::form_to_term;
use crate::number_literal::parse_number_literal;

/// Parse MATLAB text into a [`MatlabForm`] (no evaluate, no arena write).
pub fn parse_matlab_form(input: &str) -> Result<MatlabForm, SxoError> {
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
    lower_root(&root)
}

/// Parse MATLAB text into a session arena [`TermId`] (no evaluate).
///
/// Transitional: `parse_matlab_form` → [`form_to_term`]. Prefer Form APIs for new paths.
pub fn parse_matlab(session: &mut Session, input: &str) -> Result<TermId, SxoError> {
    let form = parse_matlab_form(input)?;
    Ok(form_to_term(session, &form))
}

fn lower_root(root: &MatlabRoot) -> Result<MatlabForm, SxoError> {
    let mut items = Vec::with_capacity(root.items.len());
    for stmt in &root.items {
        items.push(lower_stmt(stmt)?);
    }
    match items.len() {
        0 => Err(SxoError::new("matlab(oak): empty root")),
        1 => Ok(items.remove(0)),
        _ => Ok(MatlabForm::call("CompoundExpression", items)),
    }
}

fn lower_stmt(stmt: &Statement) -> Result<MatlabForm, SxoError> {
    match stmt {
        Statement::Expr(expr) => lower_expr(expr),
        Statement::If { condition, then_body, elseifs, else_body, .. } => {
            let mut else_form = compound_stmts(else_body)?;
            for (cond, body) in elseifs.iter().rev() {
                let then_f = compound_stmts(body)?;
                let cond_f = lower_expr(cond)?;
                else_form = MatlabForm::call("If", vec![cond_f, then_f, else_form]);
            }
            let then_f = compound_stmts(then_body)?;
            let cond_f = lower_expr(condition)?;
            let else_is_null = matches!(else_form, MatlabForm::Atom(MatlabAtom::Null));
            if else_is_null && elseifs.is_empty() {
                Ok(MatlabForm::call("If", vec![cond_f, then_f]))
            }
            else {
                Ok(MatlabForm::call("If", vec![cond_f, then_f, else_form]))
            }
        }
        Statement::While { condition, body, .. } => {
            let cond_f = lower_expr(condition)?;
            let body_f = compound_stmts(body)?;
            Ok(MatlabForm::call("While", vec![cond_f, body_f]))
        }
        Statement::For { header, body, .. } => {
            let header_f = lower_expr(header)?;
            let body_f = compound_stmts(body)?;
            if header_f.head_name() == Some("Set") {
                if let MatlabForm::Call { args, .. } = &header_f {
                    if args.len() == 2 {
                        return Ok(MatlabForm::call(
                            "For",
                            vec![args[0].clone(), args[1].clone(), body_f],
                        ));
                    }
                }
            }
            Ok(MatlabForm::call("For", vec![MatlabForm::symbol("_"), header_f, body_f]))
        }
        Statement::Try { body, catch_body, .. } => {
            let body_f = compound_stmts(body)?;
            let catch_f = compound_stmts(catch_body)?;
            Ok(MatlabForm::call("Try", vec![body_f, catch_f]))
        }
        Statement::Error { .. } => Err(SxoError::new("matlab(oak): error node")),
    }
}

fn compound_stmts(stmts: &[Statement]) -> Result<MatlabForm, SxoError> {
    let mut items = Vec::with_capacity(stmts.len());
    for s in stmts {
        items.push(lower_stmt(s)?);
    }
    Ok(compound_or_single(items))
}

fn compound_or_single(mut items: Vec<MatlabForm>) -> MatlabForm {
    match items.len() {
        0 => MatlabForm::null(),
        1 => items.remove(0),
        _ => MatlabForm::call("CompoundExpression", items),
    }
}

fn lower_expr(expr: &Expression) -> Result<MatlabForm, SxoError> {
    match expr {
        Expression::Symbol(id) => {
            if id.name == "end" {
                Ok(MatlabForm::symbol("end"))
            }
            else if id.name == "true" {
                Ok(MatlabForm::bool(true))
            }
            else if id.name == "false" {
                Ok(MatlabForm::bool(false))
            }
            else {
                Ok(MatlabForm::symbol(&id.name))
            }
        }
        Expression::Literal { value, .. } => {
            let text = value.trim();
            if let Some(n) = parse_number_literal(text) {
                return Ok(MatlabForm::number(n));
            }
            if (text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
            {
                Ok(MatlabForm::string(text[1..text.len() - 1].to_string()))
            }
            else {
                Ok(MatlabForm::symbol(text))
            }
        }
        Expression::Array { rows, .. } => {
            if rows.len() == 1 {
                let mut items = Vec::with_capacity(rows[0].len());
                for cell in &rows[0] {
                    items.push(lower_expr(cell)?);
                }
                Ok(MatlabForm::list(items))
            }
            else {
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    let mut cols = Vec::with_capacity(row.len());
                    for cell in row {
                        cols.push(lower_expr(cell)?);
                    }
                    out.push(MatlabForm::list(cols));
                }
                Ok(MatlabForm::list(out))
            }
        }
        Expression::Call { head, arguments, .. } => {
            let mut head_f = lower_expr(head)?;
            if let MatlabForm::Atom(MatlabAtom::Symbol(name)) = &head_f {
                head_f = MatlabForm::symbol(map_matlab_head(name));
            }
            let mut args = Vec::with_capacity(arguments.len());
            for a in arguments {
                args.push(lower_expr(a)?);
            }
            let is_part_base = matches!(head_f, MatlabForm::List(_)) || head_f.head_name() == Some("Part");
            if is_part_base {
                let mut part_args = vec![head_f];
                part_args.extend(args);
                Ok(MatlabForm::call("Part", part_args))
            }
            else if let MatlabForm::Atom(MatlabAtom::Symbol(name)) = &head_f {
                Ok(MatlabForm::call(name.clone(), args))
            }
            else {
                // Non-symbol head → Application[head, args…] (form_to_term → ApplyHead).
                let mut wrapped = vec![head_f];
                wrapped.extend(args);
                Ok(MatlabForm::call("Application", wrapped))
            }
        }
        Expression::Binary(bin) => lower_binary(bin),
        Expression::Prefix(u) => lower_prefix(u),
        Expression::Postfix(u) => lower_postfix(u),
        Expression::Grouped { expression, .. } => lower_expr(expression),
    }
}

fn lower_binary(bin: &BinaryExpr) -> Result<MatlabForm, SxoError> {
    let l = lower_expr(&bin.lhs)?;
    let r = lower_expr(&bin.rhs)?;
    Ok(match bin.operator {
        MatlabTokenType::Plus => MatlabForm::call("Plus", vec![l, r]),
        MatlabTokenType::Minus => MatlabForm::call("Subtract", vec![l, r]),
        MatlabTokenType::Times => MatlabForm::call("Times", vec![l, r]),
        MatlabTokenType::DotTimes => MatlabForm::call("DotTimes", vec![l, r]),
        MatlabTokenType::Divide => MatlabForm::call("Divide", vec![l, r]),
        MatlabTokenType::DotDivide => MatlabForm::call("DotDivide", vec![l, r]),
        MatlabTokenType::LeftDivide => MatlabForm::call("LinearSolve", vec![l, r]),
        MatlabTokenType::DotLeftDivide => MatlabForm::call("DotLeftDivide", vec![l, r]),
        MatlabTokenType::Power => MatlabForm::call("Power", vec![l, r]),
        MatlabTokenType::DotPower => MatlabForm::call("DotPower", vec![l, r]),
        MatlabTokenType::Assign => MatlabForm::call("Set", vec![l, r]),
        MatlabTokenType::Equal => MatlabForm::call("Equal", vec![l, r]),
        MatlabTokenType::NotEqual => MatlabForm::call("Unequal", vec![l, r]),
        MatlabTokenType::Less => MatlabForm::call("Less", vec![l, r]),
        MatlabTokenType::Greater => MatlabForm::call("Greater", vec![l, r]),
        MatlabTokenType::LessEqual => MatlabForm::call("LessEqual", vec![l, r]),
        MatlabTokenType::GreaterEqual => MatlabForm::call("GreaterEqual", vec![l, r]),
        MatlabTokenType::AndAnd | MatlabTokenType::And => MatlabForm::call("And", vec![l, r]),
        MatlabTokenType::OrOr | MatlabTokenType::Or => MatlabForm::call("Or", vec![l, r]),
        MatlabTokenType::Colon => flatten_range(l, r),
        other => {
            return Err(SxoError::new(format!("matlab(ast): unsupported binary {other:?}")));
        }
    })
}

fn flatten_range(left: MatlabForm, right: MatlabForm) -> MatlabForm {
    if left.head_name() == Some("Range") {
        if let MatlabForm::Call { args, .. } = &left {
            if args.len() == 2 {
                // MATLAB `start:step:end` → Athena `Range[start, end, step]`.
                return MatlabForm::call("Range", vec![args[0].clone(), right, args[1].clone()]);
            }
        }
    }
    MatlabForm::call("Range", vec![left, right])
}

fn lower_prefix(u: &UnaryExpr) -> Result<MatlabForm, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        MatlabTokenType::Minus => MatlabForm::call("Minus", vec![e]),
        MatlabTokenType::Plus => e,
        MatlabTokenType::Not => MatlabForm::call("Not", vec![e]),
        other => return Err(SxoError::new(format!("matlab(ast): unsupported prefix {other:?}"))),
    })
}

fn lower_postfix(u: &UnaryExpr) -> Result<MatlabForm, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        MatlabTokenType::Transpose | MatlabTokenType::DotTranspose => MatlabForm::call("Transpose", vec![e]),
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
