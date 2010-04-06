//! Mathematica / Wolfram dialect via oaks **language AST** (`WolframBuilder`) → [`WolframForm`].
//!
//! Formal path: oak CST → [`WolframRoot`] / [`Expression`] → dialect Form.
//! Do not expand GreenTree / `WolframElementType` layout walks here.

use oak_core::{Builder, source::SourceText};
use oak_wolfram::{
    WolframBuilder, WolframLanguage,
    ast::{BinaryExpr, Expression, UnaryExpr, WolframRoot},
    lexer::token_type::WolframTokenType,
};

use sxo_types::SxoError;

use crate::{
    form::{WolframAtom, WolframForm},
    number_literal::parse_number_literal,
};

/// Parse Mathematica / Wolfram text into a structural [`WolframForm`] (no evaluate).
pub fn parse_mathematica(input: &str) -> Result<WolframForm, SxoError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SxoError::new("mathematica: empty input"));
    }

    let language = WolframLanguage::default();
    let builder = WolframBuilder::new(&language);
    let source = SourceText::new(trimmed);
    let mut session = oak_core::ParseSession::<WolframLanguage>::default();
    let output = builder.build(&source, &[], &mut session);
    let root = output.result.map_err(|e| SxoError::new(format!("mathematica(ast): {e:?}")))?;
    lower_root(&root)
}

fn lower_root(root: &WolframRoot) -> Result<WolframForm, SxoError> {
    let mut items = Vec::with_capacity(root.expressions.len());
    for expr in &root.expressions {
        if matches!(expr, Expression::Error { .. }) {
            continue;
        }
        items.push(lower_expr(expr)?);
    }
    match items.len() {
        0 => Err(SxoError::new("mathematica(ast): empty root")),
        1 => Ok(items.remove(0)),
        _ => Ok(WolframForm::call("CompoundExpression", items)),
    }
}

fn lower_expr(expr: &Expression) -> Result<WolframForm, SxoError> {
    match expr {
        Expression::Symbol(id) => lower_symbol_name(&id.name),
        Expression::Literal { value, .. } => lower_literal(value),
        Expression::List { elements, .. } => {
            let mut items = Vec::with_capacity(elements.len());
            for e in elements {
                items.push(lower_expr(e)?);
            }
            Ok(WolframForm::List(items))
        }
        Expression::Call { head, arguments, .. } => {
            let head_w = lower_expr(head)?;
            let mut args = Vec::with_capacity(arguments.len());
            for a in arguments {
                args.push(lower_expr(a)?);
            }
            if head_w.is_symbol("List") {
                return Ok(WolframForm::List(args));
            }
            Ok(WolframForm::Call { head: Box::new(head_w), args })
        }
        Expression::Part { expression, indices, .. } => {
            let mut args = vec![lower_expr(expression)?];
            for i in indices {
                args.push(lower_expr(i)?);
            }
            Ok(WolframForm::call("Part", args))
        }
        Expression::Binary(bin) => lower_binary(bin),
        Expression::Prefix(u) => lower_prefix(u),
        Expression::Postfix(u) => lower_postfix(u),
        Expression::Blank { kind, head, .. } => lower_blank(*kind, head.as_deref()),
        Expression::Pattern { name, blank, .. } => {
            let name_w = lower_expr(name)?;
            let blank_w = lower_blank(*blank, None)?;
            Ok(WolframForm::call("Pattern", vec![name_w, blank_w]))
        }
        Expression::Grouped { expression, .. } => lower_expr(expression),
        Expression::Error { .. } => Err(SxoError::new("mathematica(ast): error node")),
    }
}

fn lower_symbol_name(name: &str) -> Result<WolframForm, SxoError> {
    // oaks finishes Slot tokens as Symbol nodes whose text is `#` / `#n`.
    if name == "#" || name == "#1" {
        return Ok(WolframForm::call("Slot", vec![WolframForm::int(1)]));
    }
    if let Some(rest) = name.strip_prefix('#') {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = rest.parse::<i64>() {
                return Ok(WolframForm::call("Slot", vec![WolframForm::int(n)]));
            }
        }
    }
    Ok(WolframForm::symbol(name))
}

fn lower_literal(value: &str) -> Result<WolframForm, SxoError> {
    let text = value.trim();
    if let Some(n) = parse_number_literal(text) {
        return Ok(WolframForm::number(n));
    }
    if text.starts_with('"') {
        Ok(WolframForm::Atom(WolframAtom::String(text.trim_matches('"').to_string())))
    }
    else {
        Err(SxoError::new(format!("mathematica(ast): bad literal `{text}`")))
    }
}

fn lower_binary(bin: &BinaryExpr) -> Result<WolframForm, SxoError> {
    let l = lower_expr(&bin.lhs)?;
    let r = lower_expr(&bin.rhs)?;
    Ok(match bin.operator {
        WolframTokenType::Plus => WolframForm::call("Plus", vec![l, r]),
        WolframTokenType::Minus => WolframForm::call("Subtract", vec![l, r]),
        WolframTokenType::Times => WolframForm::call("Times", vec![l, r]),
        WolframTokenType::Divide => WolframForm::call("Divide", vec![l, r]),
        WolframTokenType::Power => WolframForm::call("Power", vec![l, r]),
        WolframTokenType::At => WolframForm::Call { head: Box::new(l), args: vec![r] },
        WolframTokenType::SlashSlash => WolframForm::Call { head: Box::new(r), args: vec![l] },
        WolframTokenType::Arrow | WolframTokenType::Rule => WolframForm::call("Rule", vec![l, r]),
        WolframTokenType::RuleDelayedOp | WolframTokenType::RuleDelayed | WolframTokenType::DoubleArrow => {
            WolframForm::call("RuleDelayed", vec![l, r])
        }
        WolframTokenType::MapOperator => WolframForm::call("Map", vec![l, r]),
        WolframTokenType::ApplyOperator => WolframForm::call("Apply", vec![l, r]),
        WolframTokenType::ApplyLevelOperator => {
            WolframForm::call("Apply", vec![l, r, WolframForm::List(vec![WolframForm::int(1)])])
        }
        WolframTokenType::MapAllOperator => WolframForm::call("MapAll", vec![l, r]),
        WolframTokenType::Semicolon => WolframForm::call("CompoundExpression", vec![l, r]),
        WolframTokenType::Assign | WolframTokenType::Set => WolframForm::call("Set", vec![l, r]),
        WolframTokenType::SetDelayed => WolframForm::call("SetDelayed", vec![l, r]),
        WolframTokenType::Equal => WolframForm::call("Equal", vec![l, r]),
        WolframTokenType::NotEqual => WolframForm::call("Unequal", vec![l, r]),
        WolframTokenType::Less => WolframForm::call("Less", vec![l, r]),
        WolframTokenType::Greater => WolframForm::call("Greater", vec![l, r]),
        WolframTokenType::LessEqual => WolframForm::call("LessEqual", vec![l, r]),
        WolframTokenType::GreaterEqual => WolframForm::call("GreaterEqual", vec![l, r]),
        WolframTokenType::And => WolframForm::call("And", vec![l, r]),
        WolframTokenType::Or => WolframForm::call("Or", vec![l, r]),
        other => return Err(SxoError::new(format!("mathematica(ast): unsupported binary {other:?}"))),
    })
}

fn lower_prefix(u: &UnaryExpr) -> Result<WolframForm, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        WolframTokenType::Minus => WolframForm::call("Times", vec![WolframForm::int(-1), e]),
        WolframTokenType::Factorial => WolframForm::call("Not", vec![e]),
        other => return Err(SxoError::new(format!("mathematica(ast): unsupported prefix {other:?}"))),
    })
}

fn lower_postfix(u: &UnaryExpr) -> Result<WolframForm, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        WolframTokenType::Ampersand => WolframForm::call("Function", vec![e]),
        WolframTokenType::Factorial => WolframForm::call("Factorial", vec![e]),
        WolframTokenType::Underscore => WolframForm::call("Pattern", vec![e, WolframForm::call("Blank", vec![])]),
        WolframTokenType::DoubleUnderscore => WolframForm::call("Pattern", vec![e, WolframForm::call("BlankSequence", vec![])]),
        WolframTokenType::TripleUnderscore => {
            WolframForm::call("Pattern", vec![e, WolframForm::call("BlankNullSequence", vec![])])
        }
        other => return Err(SxoError::new(format!("mathematica(ast): unsupported postfix {other:?}"))),
    })
}

fn lower_blank(kind: WolframTokenType, head: Option<&Expression>) -> Result<WolframForm, SxoError> {
    let blank_head = match kind {
        WolframTokenType::DoubleUnderscore => "BlankSequence",
        WolframTokenType::TripleUnderscore => "BlankNullSequence",
        _ => "Blank",
    };
    Ok(match head {
        Some(h) => WolframForm::call(blank_head, vec![lower_expr(h)?]),
        None => WolframForm::call(blank_head, vec![]),
    })
}
