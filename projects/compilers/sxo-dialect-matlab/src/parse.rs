//! MATLAB dialect via oaks **language AST** (`MatlabBuilder`) → engine [`Term`].
//!
//! Formal path: oak CST → [`MatlabRoot`] / [`Statement`] / [`Expression`] → Term.
//! Do not expand GreenTree / `MatlabTokenType` leaf walking here.

use oak_core::{Builder, source::SourceText};
use oak_matlab::{
    MatlabBuilder, MatlabLanguage,
    ast::{BinaryExpr, Expression, MatlabRoot, Statement, UnaryExpr},
    lexer::token_type::MatlabTokenType,
};

use athena::{Atom, Term};
use sxo_types::SxoError;

use crate::shared::term_from_number_literal;

/// Parse MATLAB text into engine [`Term`] (no evaluate).
pub fn parse_matlab(input: &str) -> Result<Term, SxoError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SxoError::new("matlab: empty input"));
    }

    let language = MatlabLanguage::default();
    let builder = MatlabBuilder::new(&language);
    let source = SourceText::new(trimmed);
    let mut session = oak_core::ParseSession::<MatlabLanguage>::default();
    let output = builder.build(&source, &[], &mut session);
    let root = output.result.map_err(|e| SxoError::new(format!("matlab(oak): {e:?}")))?;
    lower_root(&root)
}

fn lower_root(root: &MatlabRoot) -> Result<Term, SxoError> {
    let mut items = Vec::with_capacity(root.items.len());
    for stmt in &root.items {
        items.push(lower_stmt(stmt)?);
    }
    match items.len() {
        0 => Err(SxoError::new("matlab(oak): empty root")),
        1 => Ok(items.remove(0)),
        _ => Ok(Term::apply("CompoundExpression", items)),
    }
}

fn lower_stmt(stmt: &Statement) -> Result<Term, SxoError> {
    match stmt {
        Statement::Expr(expr) => lower_expr(expr),
        Statement::If { condition, then_body, elseifs, else_body, .. } => {
            // Flatten elseif into nested If for Athena.
            let mut else_term = compound_stmts(else_body)?;
            for (cond, body) in elseifs.iter().rev() {
                let then_t = compound_stmts(body)?;
                else_term = Term::apply("If", vec![lower_expr(cond)?, then_t, else_term]);
            }
            let then_t = compound_stmts(then_body)?;
            if matches!(&else_term, Term::Atom(Atom::Null)) && elseifs.is_empty() {
                Ok(Term::apply("If", vec![lower_expr(condition)?, then_t]))
            }
            else {
                Ok(Term::apply("If", vec![lower_expr(condition)?, then_t, else_term]))
            }
        }
        Statement::While { condition, body, .. } => {
            Ok(Term::apply("While", vec![lower_expr(condition)?, compound_stmts(body)?]))
        }
        Statement::For { header, body, .. } => {
            let header_t = lower_expr(header)?;
            let body_t = compound_stmts(body)?;
            match header_t {
                Term::Application { head, arguments: args } if head.is_symbol("Set") && args.len() == 2 => {
                    Ok(Term::apply("For", vec![athena::clone_term(&args[0]), athena::clone_term(&args[1]), body_t]))
                }
                other => Ok(Term::apply("For", vec![Term::symbol("_"), other, body_t])),
            }
        }
        Statement::Try { body, catch_body, .. } => {
            // Athena `Try[body, catch]`. catch_name is ignored in this slice.
            Ok(Term::apply("Try", vec![compound_stmts(body)?, compound_stmts(catch_body)?]))
        }
        Statement::Error { .. } => Err(SxoError::new("matlab(oak): error node")),
    }
}

fn compound_stmts(stmts: &[Statement]) -> Result<Term, SxoError> {
    let mut items = Vec::with_capacity(stmts.len());
    for s in stmts {
        items.push(lower_stmt(s)?);
    }
    Ok(compound_or_single(items))
}

fn compound_or_single(mut items: Vec<Term>) -> Term {
    match items.len() {
        0 => Term::null(),
        1 => items.remove(0),
        _ => Term::apply("CompoundExpression", items),
    }
}

fn lower_expr(expr: &Expression) -> Result<Term, SxoError> {
    match expr {
        Expression::Symbol(id) => {
            if id.name == "end" {
                Ok(Term::symbol("End"))
            }
            else if id.name == "true" {
                Ok(Term::boolean(true))
            }
            else if id.name == "false" {
                Ok(Term::boolean(false))
            }
            else {
                Ok(Term::symbol(&id.name))
            }
        }
        Expression::Literal { value, .. } => {
            let text = value.trim();
            if let Some(t) = term_from_number_literal(text) {
                return Ok(t);
            }
            if (text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
            {
                Ok(Term::Atom(Atom::String(text[1..text.len() - 1].to_string())))
            }
            else {
                Ok(Term::symbol(text))
            }
        }
        Expression::Array { rows, .. } => {
            if rows.len() == 1 {
                Ok(Term::List(rows[0].iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?))
            }
            else {
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    out.push(Term::List(row.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?));
                }
                Ok(Term::List(out))
            }
        }
        Expression::Call { head, arguments, .. } => {
            let mut expr_t = lower_expr(head)?;
            // Map bare symbol call heads (sin → Sin).
            if let Term::Atom(Atom::Symbol(name)) = &expr_t {
                expr_t = Term::symbol(map_matlab_head(name));
            }
            let args = arguments.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;
            if matches!(&expr_t, Term::List(_))
                || matches!(&expr_t, Term::Application { head: h, .. } if h.is_symbol("Part"))
            {
                let mut part_args = vec![expr_t];
                part_args.extend(args);
                Ok(Term::apply("Part", part_args))
            }
            else {
                Ok(Term::Application { head: Box::new(expr_t), arguments: args })
            }
        }
        Expression::Binary(bin) => lower_binary(bin),
        Expression::Prefix(u) => lower_prefix(u),
        Expression::Postfix(u) => lower_postfix(u),
        Expression::Grouped { expression, .. } => lower_expr(expression),
    }
}

fn lower_binary(bin: &BinaryExpr) -> Result<Term, SxoError> {
    let l = lower_expr(&bin.lhs)?;
    let r = lower_expr(&bin.rhs)?;
    Ok(match bin.operator {
        MatlabTokenType::Plus => Term::apply("Plus", vec![l, r]),
        MatlabTokenType::Minus => Term::apply("Subtract", vec![l, r]),
        MatlabTokenType::Times => Term::apply("Times", vec![l, r]),
        MatlabTokenType::DotTimes => Term::apply("DotTimes", vec![l, r]),
        MatlabTokenType::Divide => Term::apply("Divide", vec![l, r]),
        MatlabTokenType::DotDivide => Term::apply("DotDivide", vec![l, r]),
        MatlabTokenType::LeftDivide => Term::apply("Mldivide", vec![l, r]),
        MatlabTokenType::DotLeftDivide => Term::apply("DotLeftDivide", vec![l, r]),
        MatlabTokenType::Power => Term::apply("Power", vec![l, r]),
        MatlabTokenType::DotPower => Term::apply("DotPower", vec![l, r]),
        MatlabTokenType::Assign => Term::apply("Set", vec![l, r]),
        MatlabTokenType::Equal => Term::apply("Equal", vec![l, r]),
        MatlabTokenType::NotEqual => Term::apply("Unequal", vec![l, r]),
        MatlabTokenType::Less => Term::apply("Less", vec![l, r]),
        MatlabTokenType::Greater => Term::apply("Greater", vec![l, r]),
        MatlabTokenType::LessEqual => Term::apply("LessEqual", vec![l, r]),
        MatlabTokenType::GreaterEqual => Term::apply("GreaterEqual", vec![l, r]),
        MatlabTokenType::AndAnd | MatlabTokenType::And => Term::apply("And", vec![l, r]),
        MatlabTokenType::OrOr | MatlabTokenType::Or => Term::apply("Or", vec![l, r]),
        MatlabTokenType::Colon => flatten_span(l, r),
        other => {
            return Err(SxoError::new(format!("matlab(ast): unsupported binary {other:?}")));
        }
    })
}

fn flatten_span(left: Term, right: Term) -> Term {
    // `1:2:10` parses as Colon(Colon(1,2), 10) → Span[1,2,10]
    match left {
        Term::Application { head, arguments: args } if head.is_symbol("Span") && args.len() == 2 => {
            Term::apply("Span", vec![athena::clone_term(&args[0]), athena::clone_term(&args[1]), right])
        }
        other => Term::apply("Span", vec![other, right]),
    }
}

fn lower_prefix(u: &UnaryExpr) -> Result<Term, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        MatlabTokenType::Minus => Term::apply("Times", vec![Term::int(-1), e]),
        MatlabTokenType::Plus => e,
        MatlabTokenType::Not => Term::apply("Not", vec![e]),
        other => return Err(SxoError::new(format!("matlab(ast): unsupported prefix {other:?}"))),
    })
}

fn lower_postfix(u: &UnaryExpr) -> Result<Term, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        MatlabTokenType::Transpose | MatlabTokenType::DotTranspose => Term::apply("Transpose", vec![e]),
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
