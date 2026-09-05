//! Mathematica / Wolfram dialect via oaks **language AST** (`WolframBuilder`) → [`WExpr`].
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
    form::{WAtom, WExpr},
    shared::parse_number_literal,
};

/// Parse Mathematica / Wolfram text into a structural [`WExpr`] (no evaluate).
pub fn parse_mathematica(input: &str) -> Result<WExpr, SxoError> {
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

fn lower_root(root: &WolframRoot) -> Result<WExpr, SxoError> {
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
        _ => Ok(WExpr::call("CompoundExpression", items)),
    }
}

fn lower_expr(expr: &Expression) -> Result<WExpr, SxoError> {
    match expr {
        Expression::Symbol(id) => lower_symbol_name(&id.name),
        Expression::Literal { value, .. } => lower_literal(value),
        Expression::List { elements, .. } => {
            let mut items = Vec::with_capacity(elements.len());
            for e in elements {
                items.push(lower_expr(e)?);
            }
            Ok(WExpr::List(items))
        }
        Expression::Call { head, arguments, .. } => {
            let head_w = lower_expr(head)?;
            let mut args = Vec::with_capacity(arguments.len());
            for a in arguments {
                args.push(lower_expr(a)?);
            }
            if head_w.is_symbol("List") {
                return Ok(WExpr::List(args));
            }
            Ok(WExpr::Call { head: Box::new(head_w), args })
        }
        Expression::Part { expression, indices, .. } => {
            let mut args = vec![lower_expr(expression)?];
            for i in indices {
                args.push(lower_expr(i)?);
            }
            Ok(WExpr::call("Part", args))
        }
        Expression::Binary(bin) => lower_binary(bin),
        Expression::Prefix(u) => lower_prefix(u),
        Expression::Postfix(u) => lower_postfix(u),
        Expression::Blank { kind, head, .. } => lower_blank(*kind, head.as_deref()),
        Expression::Pattern { name, blank, .. } => {
            let name_w = lower_expr(name)?;
            let blank_w = lower_blank(*blank, None)?;
            Ok(WExpr::call("Pattern", vec![name_w, blank_w]))
        }
        Expression::Grouped { expression, .. } => lower_expr(expression),
        Expression::Error { .. } => Err(SxoError::new("mathematica(ast): error node")),
    }
}

fn lower_symbol_name(name: &str) -> Result<WExpr, SxoError> {
    // oaks finishes Slot tokens as Symbol nodes whose text is `#` / `#n`.
    if name == "#" || name == "#1" {
        return Ok(WExpr::call("Slot", vec![WExpr::int(1)]));
    }
    if let Some(rest) = name.strip_prefix('#') {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = rest.parse::<i64>() {
                return Ok(WExpr::call("Slot", vec![WExpr::int(n)]));
            }
        }
    }
    Ok(WExpr::symbol(name))
}

fn lower_literal(value: &str) -> Result<WExpr, SxoError> {
    let text = value.trim();
    if let Some(n) = parse_number_literal(text) {
        return Ok(WExpr::number(n));
    }
    if text.starts_with('"') {
        Ok(WExpr::Atom(WAtom::String(text.trim_matches('"').to_string())))
    }
    else {
        Err(SxoError::new(format!("mathematica(ast): bad literal `{text}`")))
    }
}

fn lower_binary(bin: &BinaryExpr) -> Result<WExpr, SxoError> {
    let l = lower_expr(&bin.lhs)?;
    let r = lower_expr(&bin.rhs)?;
    Ok(match bin.operator {
        WolframTokenType::Plus => WExpr::call("Plus", vec![l, r]),
        WolframTokenType::Minus => WExpr::call("Subtract", vec![l, r]),
        WolframTokenType::Times => WExpr::call("Times", vec![l, r]),
        WolframTokenType::Divide => WExpr::call("Divide", vec![l, r]),
        WolframTokenType::Power => WExpr::call("Power", vec![l, r]),
        WolframTokenType::At => WExpr::Call { head: Box::new(l), args: vec![r] },
        WolframTokenType::SlashSlash => WExpr::Call { head: Box::new(r), args: vec![l] },
        WolframTokenType::Arrow | WolframTokenType::Rule => WExpr::call("Rule", vec![l, r]),
        WolframTokenType::RuleDelayedOp | WolframTokenType::RuleDelayed | WolframTokenType::DoubleArrow => {
            WExpr::call("RuleDelayed", vec![l, r])
        }
        WolframTokenType::MapOperator => WExpr::call("Map", vec![l, r]),
        WolframTokenType::ApplyOperator => WExpr::call("Apply", vec![l, r]),
        WolframTokenType::ApplyLevelOperator => WExpr::call("Apply", vec![l, r, WExpr::List(vec![WExpr::int(1)])]),
        WolframTokenType::MapAllOperator => WExpr::call("MapAll", vec![l, r]),
        WolframTokenType::Semicolon => WExpr::call("CompoundExpression", vec![l, r]),
        WolframTokenType::Assign | WolframTokenType::Set => WExpr::call("Set", vec![l, r]),
        WolframTokenType::SetDelayed => WExpr::call("SetDelayed", vec![l, r]),
        WolframTokenType::Equal => WExpr::call("Equal", vec![l, r]),
        WolframTokenType::NotEqual => WExpr::call("Unequal", vec![l, r]),
        WolframTokenType::Less => WExpr::call("Less", vec![l, r]),
        WolframTokenType::Greater => WExpr::call("Greater", vec![l, r]),
        WolframTokenType::LessEqual => WExpr::call("LessEqual", vec![l, r]),
        WolframTokenType::GreaterEqual => WExpr::call("GreaterEqual", vec![l, r]),
        WolframTokenType::And => WExpr::call("And", vec![l, r]),
        WolframTokenType::Or => WExpr::call("Or", vec![l, r]),
        other => return Err(SxoError::new(format!("mathematica(ast): unsupported binary {other:?}"))),
    })
}

fn lower_prefix(u: &UnaryExpr) -> Result<WExpr, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        WolframTokenType::Minus => WExpr::call("Times", vec![WExpr::int(-1), e]),
        WolframTokenType::Factorial => WExpr::call("Not", vec![e]),
        other => return Err(SxoError::new(format!("mathematica(ast): unsupported prefix {other:?}"))),
    })
}

fn lower_postfix(u: &UnaryExpr) -> Result<WExpr, SxoError> {
    let e = lower_expr(&u.operand)?;
    Ok(match u.operator {
        WolframTokenType::Ampersand => WExpr::call("Function", vec![e]),
        WolframTokenType::Factorial => WExpr::call("Factorial", vec![e]),
        WolframTokenType::Underscore => WExpr::call("Pattern", vec![e, WExpr::call("Blank", vec![])]),
        WolframTokenType::DoubleUnderscore => WExpr::call("Pattern", vec![e, WExpr::call("BlankSequence", vec![])]),
        WolframTokenType::TripleUnderscore => WExpr::call("Pattern", vec![e, WExpr::call("BlankNullSequence", vec![])]),
        other => return Err(SxoError::new(format!("mathematica(ast): unsupported postfix {other:?}"))),
    })
}

fn lower_blank(kind: WolframTokenType, head: Option<&Expression>) -> Result<WExpr, SxoError> {
    let blank_head = match kind {
        WolframTokenType::DoubleUnderscore => "BlankSequence",
        WolframTokenType::TripleUnderscore => "BlankNullSequence",
        _ => "Blank",
    };
    Ok(match head {
        Some(h) => WExpr::call(blank_head, vec![lower_expr(h)?]),
        None => WExpr::call(blank_head, vec![]),
    })
}
