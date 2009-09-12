//! Mathematica / Wolfram dialect via **oaks** `oak-wolfram` → [`WExpr`].

use oak_core::{
    Parser,
    parser::ParseSession,
    source::SourceText,
    tree::{GreenNode, GreenTree},
};
use oak_wolfram::{
    WolframLanguage, WolframParser, lexer::token_type::WolframTokenType, parser::element_type::WolframElementType,
};

use sxo_types::SxoError;

use crate::{
    mma_bridge::term_to_wexpr,
    number_literal::term_from_number_literal,
    wexpr::{WAtom, WExpr},
};

/// Parse Mathematica / Wolfram text into a structural [`WExpr`] (no evaluate).
pub fn parse_mathematica(input: &str) -> Result<WExpr, SxoError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SxoError::new("mathematica: empty input"));
    }

    let language = WolframLanguage::default();
    let parser = WolframParser::new(&language);
    let source = SourceText::new(trimmed);
    let mut session = ParseSession::<WolframLanguage>::default();
    let output = parser.parse(&source, &[], &mut session);

    let root = output.result.map_err(|e| SxoError::new(format!("mathematica(oak): {e:?}")))?;
    lower_root(root, trimmed)
}

fn lower_root(root: &GreenNode<'_, WolframLanguage>, src: &str) -> Result<WExpr, SxoError> {
    let mut offset = 0usize;
    let mut last: Option<WExpr> = None;
    for child in root.children {
        match child {
            GreenTree::Leaf(leaf) => {
                offset += leaf.length as usize;
            }
            GreenTree::Node(node) => {
                if matches!(node.kind, WolframElementType::Error) {
                    offset += node.byte_length as usize;
                    continue;
                }
                last = Some(lower_node(node, src, offset)?);
                offset += node.byte_length as usize;
            }
        }
    }
    last.ok_or_else(|| SxoError::new("mathematica(oak): empty root"))
}

fn lower_node(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<WExpr, SxoError> {
    match node.kind {
        WolframElementType::Root | WolframElementType::Expression => {
            let mut offset = start;
            for child in node.children {
                match child {
                    GreenTree::Leaf(leaf) => offset += leaf.length as usize,
                    GreenTree::Node(n) => return lower_node(n, src, offset),
                }
            }
            Err(SxoError::new("mathematica(oak): empty Expression"))
        }
        WolframElementType::Literal => {
            let text = slice(src, start, node.byte_length)?.trim();
            if let Some(t) = term_from_number_literal(text) {
                return Ok(term_to_wexpr(&t));
            }
            if text.starts_with('"') {
                Ok(WExpr::Atom(WAtom::String(text.trim_matches('"').to_string())))
            }
            else {
                Err(SxoError::new(format!("mathematica(oak): bad literal `{text}`")))
            }
        }
        WolframElementType::Symbol => {
            let name = slice(src, start, node.byte_length)?.trim().to_string();
            Ok(WExpr::symbol(name))
        }
        WolframElementType::List => lower_list(node, src, start),
        WolframElementType::PrefixExpr => lower_prefix(node, src, start),
        WolframElementType::BinaryExpr => lower_binary(node, src, start),
        WolframElementType::PostfixExpr => lower_postfix(node, src, start),
        WolframElementType::Call => lower_call(node, src, start),
        WolframElementType::Arguments => Err(SxoError::new("mathematica(oak): unexpected Arguments")),
        WolframElementType::Error => Err(SxoError::new("mathematica(oak): error node")),
    }
}

fn lower_list(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<WExpr, SxoError> {
    let mut offset = start;
    let mut items = Vec::new();
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => offset += leaf.length as usize,
            GreenTree::Node(n) => {
                if matches!(n.kind, WolframElementType::Error) {
                    offset += n.byte_length as usize;
                    continue;
                }
                items.push(lower_node(n, src, offset)?);
                offset += n.byte_length as usize;
            }
        }
    }
    Ok(WExpr::List(items))
}

fn lower_prefix(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<WExpr, SxoError> {
    let mut offset = start;
    let mut op: Option<WolframTokenType> = None;
    let mut operand: Option<WExpr> = None;
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if !is_trivia(leaf.kind) && op.is_none() {
                    op = Some(leaf.kind);
                }
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                operand = Some(lower_node(n, src, offset)?);
                offset += n.byte_length as usize;
            }
        }
    }
    match (op, operand) {
        (Some(WolframTokenType::Minus), Some(e)) => Ok(WExpr::call("Times", vec![WExpr::int(-1), e])),
        (Some(WolframTokenType::Factorial), Some(e)) => Ok(WExpr::call("Not", vec![e])),
        (Some(other), _) => Err(SxoError::new(format!("mathematica(oak): unsupported prefix {other:?}"))),
        _ => Err(SxoError::new("mathematica(oak): malformed PrefixExpr")),
    }
}

fn lower_binary(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<WExpr, SxoError> {
    let mut offset = start;
    let mut left: Option<WExpr> = None;
    let mut op: Option<WolframTokenType> = None;
    let mut right: Option<WExpr> = None;
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if !is_trivia(leaf.kind) && op.is_none() && left.is_some() {
                    op = Some(leaf.kind);
                }
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                let e = lower_node(n, src, offset)?;
                if left.is_none() {
                    left = Some(e);
                }
                else {
                    right = Some(e);
                }
                offset += n.byte_length as usize;
            }
        }
    }
    let (l, op, r) = match (left, op, right) {
        (Some(l), Some(op), Some(r)) => (l, op, r),
        _ => return Err(SxoError::new("mathematica(oak): malformed BinaryExpr")),
    };
    Ok(match op {
        WolframTokenType::Plus => WExpr::call("Plus", vec![l, r]),
        WolframTokenType::Minus => WExpr::call("Subtract", vec![l, r]),
        WolframTokenType::Times => WExpr::call("Times", vec![l, r]),
        WolframTokenType::Divide => WExpr::call("Divide", vec![l, r]),
        WolframTokenType::Power => WExpr::call("Power", vec![l, r]),
        WolframTokenType::At => {
            // f @ x → f[x]
            WExpr::Call { head: Box::new(l), args: vec![r] }
        }
        WolframTokenType::SlashSlash => {
            // x // f → f[x]
            WExpr::Call { head: Box::new(r), args: vec![l] }
        }
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
        other => {
            return Err(SxoError::new(format!("mathematica(oak): unsupported binary {other:?}")));
        }
    })
}

fn lower_postfix(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<WExpr, SxoError> {
    let mut offset = start;
    let mut expr: Option<WExpr> = None;
    let mut op: Option<WolframTokenType> = None;
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if !is_trivia(leaf.kind) && matches!(leaf.kind, WolframTokenType::Ampersand | WolframTokenType::Factorial) {
                    op = Some(leaf.kind);
                }
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                expr = Some(lower_node(n, src, offset)?);
                offset += n.byte_length as usize;
            }
        }
    }
    let expr = expr.ok_or_else(|| SxoError::new("mathematica(oak): malformed PostfixExpr"))?;
    match op {
        Some(WolframTokenType::Ampersand) => Ok(WExpr::call("Function", vec![expr])),
        Some(WolframTokenType::Factorial) => Ok(WExpr::call("Factorial", vec![expr])),
        Some(other) => Err(SxoError::new(format!("mathematica(oak): unsupported postfix {other:?}"))),
        None => Ok(expr),
    }
}

fn lower_call(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<WExpr, SxoError> {
    let mut offset = start;
    let mut head: Option<WExpr> = None;
    let mut arg_groups: Vec<Vec<WExpr>> = Vec::new();

    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if matches!(leaf.kind, WolframTokenType::Identifier) && head.is_none() {
                    let name = slice(src, offset, leaf.length)?.trim().to_string();
                    head = Some(WExpr::symbol(name));
                }
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                match n.kind {
                    WolframElementType::Symbol if head.is_none() => {
                        head = Some(lower_node(n, src, offset)?);
                    }
                    WolframElementType::Arguments => {
                        arg_groups.push(lower_arguments(n, src, offset)?);
                    }
                    WolframElementType::Call if head.is_none() => {
                        head = Some(lower_node(n, src, offset)?);
                    }
                    _ if head.is_none() => {
                        head = Some(lower_node(n, src, offset)?);
                    }
                    _ => {}
                }
                offset += n.byte_length as usize;
            }
        }
    }

    let mut expr = head.ok_or_else(|| SxoError::new("mathematica(oak): call missing head"))?;
    if arg_groups.is_empty() {
        return Ok(WExpr::Call { head: Box::new(expr), args: vec![] });
    }
    // List[a,b] → List node (single bracket group only).
    if expr.is_symbol("List") && arg_groups.len() == 1 {
        return Ok(WExpr::List(arg_groups.remove(0)));
    }
    for args in arg_groups {
        expr = WExpr::Call { head: Box::new(expr), args };
    }
    Ok(expr)
}

fn lower_arguments(node: &GreenNode<'_, WolframLanguage>, src: &str, start: usize) -> Result<Vec<WExpr>, SxoError> {
    let mut offset = start;
    let mut args = Vec::new();
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                if matches!(n.kind, WolframElementType::Error) {
                    offset += n.byte_length as usize;
                    continue;
                }
                args.push(lower_node(n, src, offset)?);
                offset += n.byte_length as usize;
            }
        }
    }
    Ok(args)
}

fn slice(src: &str, start: usize, len: u32) -> Result<&str, SxoError> {
    let end = start + len as usize;
    src.get(start..end).ok_or_else(|| SxoError::new(format!("mathematica(oak): bad span {start}..{end}")))
}

fn is_trivia(kind: WolframTokenType) -> bool {
    matches!(kind, WolframTokenType::Whitespace | WolframTokenType::Newline | WolframTokenType::Comment)
}
