//! MATLAB dialect via **oaks** `oak-matlab` → engine [`Term`] (not through [`WExpr`](crate::wexpr::WExpr)).

use oak_core::{
    Parser,
    parser::ParseSession,
    source::SourceText,
    tree::{GreenNode, GreenTree},
};
use oak_matlab::{MatlabLanguage, MatlabParser, lexer::token_type::MatlabTokenType, parser::element_type::MatlabElementType};

use athena::{Atom, Term};
use sxo_types::SxoError;

use crate::number_literal::term_from_number_literal;

/// Parse MATLAB text into engine [`Term`] (no evaluate).
pub fn parse_matlab(input: &str) -> Result<Term, SxoError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SxoError::new("matlab: empty input"));
    }

    let language = MatlabLanguage::default();
    let parser = MatlabParser::new(&language);
    let source = SourceText::new(trimmed);
    let mut session = ParseSession::<MatlabLanguage>::default();
    let output = parser.parse(&source, &[], &mut session);

    let root = output.result.map_err(|e| SxoError::new(format!("matlab(oak): {e:?}")))?;
    lower_root(root, trimmed)
}

fn lower_root(root: &GreenNode<'_, MatlabLanguage>, src: &str) -> Result<Term, SxoError> {
    let mut offset = 0usize;
    let mut last: Option<Term> = None;
    for child in root.children {
        match child {
            GreenTree::Leaf(leaf) => {
                offset += leaf.length as usize;
            }
            GreenTree::Node(node) => {
                if matches!(node.kind, MatlabElementType::Error) {
                    offset += node.byte_length as usize;
                    continue;
                }
                last = Some(lower_node(node, src, offset)?);
                offset += node.byte_length as usize;
            }
        }
    }
    last.ok_or_else(|| SxoError::new("matlab(oak): empty root"))
}

fn lower_node(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Term, SxoError> {
    match node.kind {
        MatlabElementType::Root | MatlabElementType::Expression => {
            let mut offset = start;
            for child in node.children {
                match child {
                    GreenTree::Leaf(leaf) => offset += leaf.length as usize,
                    GreenTree::Node(n) => return lower_node(n, src, offset),
                }
            }
            Err(SxoError::new("matlab(oak): empty Expression"))
        }
        MatlabElementType::Literal => {
            let text = slice(src, start, node.byte_length)?.trim();
            if let Some(t) = term_from_number_literal(text) {
                return Ok(t);
            }
            if (text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
            {
                Ok(Term::Atom(Atom::String(text[1..text.len() - 1].to_string())))
            }
            else {
                Err(SxoError::new(format!("matlab(oak): bad literal `{text}`")))
            }
        }
        MatlabElementType::Symbol => {
            let name = slice(src, start, node.byte_length)?.trim().to_string();
            Ok(Term::symbol(name))
        }
        MatlabElementType::Array => lower_array(node, src, start),
        MatlabElementType::PrefixExpr => lower_prefix(node, src, start),
        MatlabElementType::BinaryExpr => lower_binary(node, src, start),
        MatlabElementType::PostfixExpr => lower_postfix(node, src, start),
        MatlabElementType::Call => lower_call(node, src, start),
        MatlabElementType::Arguments => Err(SxoError::new("matlab(oak): unexpected Arguments")),
        MatlabElementType::Error => Err(SxoError::new("matlab(oak): error node")),
    }
}

fn lower_array(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Term, SxoError> {
    let mut offset = start;
    let mut rows: Vec<Vec<Term>> = vec![vec![]];
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if leaf.kind == MatlabTokenType::Semicolon {
                    rows.push(vec![]);
                }
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                if matches!(n.kind, MatlabElementType::Error) {
                    offset += n.byte_length as usize;
                    continue;
                }
                rows.last_mut().unwrap().push(lower_node(n, src, offset)?);
                offset += n.byte_length as usize;
            }
        }
    }
    if rows.len() == 1 { Ok(Term::List(rows.remove(0))) } else { Ok(Term::List(rows.into_iter().map(Term::List).collect())) }
}

fn lower_prefix(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Term, SxoError> {
    let mut offset = start;
    let mut op: Option<MatlabTokenType> = None;
    let mut operand: Option<Term> = None;
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
        (Some(MatlabTokenType::Minus), Some(e)) => Ok(Term::app("Times", vec![Term::int(-1), e])),
        (Some(MatlabTokenType::Plus), Some(e)) => Ok(e),
        (Some(MatlabTokenType::Not), Some(e)) => Ok(Term::app("Not", vec![e])),
        (Some(other), _) => Err(SxoError::new(format!("matlab(oak): unsupported prefix {other:?}"))),
        _ => Err(SxoError::new("matlab(oak): malformed PrefixExpr")),
    }
}

fn lower_binary(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Term, SxoError> {
    let mut offset = start;
    let mut left: Option<Term> = None;
    let mut op: Option<MatlabTokenType> = None;
    let mut right: Option<Term> = None;
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
        _ => return Err(SxoError::new("matlab(oak): malformed BinaryExpr")),
    };
    Ok(match op {
        MatlabTokenType::Plus => Term::app("Plus", vec![l, r]),
        MatlabTokenType::Minus => Term::app("Subtract", vec![l, r]),
        MatlabTokenType::Times | MatlabTokenType::DotTimes => Term::app("Times", vec![l, r]),
        MatlabTokenType::Divide | MatlabTokenType::DotDivide => Term::app("Divide", vec![l, r]),
        MatlabTokenType::LeftDivide | MatlabTokenType::DotLeftDivide => Term::app("Divide", vec![r, l]),
        MatlabTokenType::Power | MatlabTokenType::DotPower => Term::app("Power", vec![l, r]),
        MatlabTokenType::Assign => Term::app("Set", vec![l, r]),
        MatlabTokenType::Equal => Term::app("Equal", vec![l, r]),
        MatlabTokenType::NotEqual => Term::app("Unequal", vec![l, r]),
        MatlabTokenType::Less => Term::app("Less", vec![l, r]),
        MatlabTokenType::Greater => Term::app("Greater", vec![l, r]),
        MatlabTokenType::LessEqual => Term::app("LessEqual", vec![l, r]),
        MatlabTokenType::GreaterEqual => Term::app("GreaterEqual", vec![l, r]),
        MatlabTokenType::AndAnd | MatlabTokenType::And => Term::app("And", vec![l, r]),
        MatlabTokenType::OrOr | MatlabTokenType::Or => Term::app("Or", vec![l, r]),
        MatlabTokenType::Colon => Term::app("Span", vec![l, r]),
        other => {
            return Err(SxoError::new(format!("matlab(oak): unsupported binary {other:?}")));
        }
    })
}

fn lower_postfix(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Term, SxoError> {
    let mut offset = start;
    let mut expr: Option<Term> = None;
    let mut op: Option<MatlabTokenType> = None;
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if !is_trivia(leaf.kind) && matches!(leaf.kind, MatlabTokenType::Transpose | MatlabTokenType::DotTranspose) {
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
    let expr = expr.ok_or_else(|| SxoError::new("matlab(oak): malformed PostfixExpr"))?;
    match op {
        Some(MatlabTokenType::Transpose) | Some(MatlabTokenType::DotTranspose) => Ok(Term::app("Transpose", vec![expr])),
        Some(other) => Err(SxoError::new(format!("matlab(oak): unsupported postfix {other:?}"))),
        None => Ok(expr),
    }
}

fn lower_call(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Term, SxoError> {
    let mut offset = start;
    let mut head: Option<Term> = None;
    let mut arg_groups: Vec<Vec<Term>> = Vec::new();

    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                if matches!(leaf.kind, MatlabTokenType::Identifier) && head.is_none() {
                    let name = slice(src, offset, leaf.length)?.trim().to_string();
                    head = Some(Term::symbol(map_matlab_head(&name)));
                }
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                match n.kind {
                    MatlabElementType::Symbol if head.is_none() => {
                        if let Term::Atom(Atom::Symbol(name)) = lower_node(n, src, offset)? {
                            head = Some(Term::symbol(map_matlab_head(&name)));
                        }
                    }
                    MatlabElementType::Arguments => {
                        arg_groups.push(lower_arguments(n, src, offset)?);
                    }
                    MatlabElementType::Call if head.is_none() => {
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

    let mut expr = head.ok_or_else(|| SxoError::new("matlab(oak): call missing head"))?;
    if arg_groups.is_empty() {
        return Ok(Term::Application { head: Box::new(expr), arguments: vec![] });
    }
    for args in arg_groups {
        expr = Term::Application { head: Box::new(expr), arguments: args };
    }
    Ok(expr)
}

fn lower_arguments(node: &GreenNode<'_, MatlabLanguage>, src: &str, start: usize) -> Result<Vec<Term>, SxoError> {
    let mut offset = start;
    let mut args = Vec::new();
    for child in node.children {
        match child {
            GreenTree::Leaf(leaf) => {
                offset += leaf.length as usize;
            }
            GreenTree::Node(n) => {
                if matches!(n.kind, MatlabElementType::Error) {
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
        other => other.to_string(),
    }
}

fn slice(src: &str, start: usize, len: u32) -> Result<&str, SxoError> {
    let end = start + len as usize;
    src.get(start..end).ok_or_else(|| SxoError::new(format!("matlab(oak): bad span {start}..{end}")))
}

fn is_trivia(kind: MatlabTokenType) -> bool {
    matches!(
        kind,
        MatlabTokenType::Whitespace | MatlabTokenType::Newline | MatlabTokenType::Comment | MatlabTokenType::BlockComment
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena::evaluate;

    #[test]
    fn parse_plus_times() {
        let t = parse_matlab("1 + 2 * 3").unwrap();
        assert_eq!(evaluate(&t), Term::int(7));
    }

    #[test]
    fn parse_array() {
        let t = parse_matlab("[1, 2 + 2]").unwrap();
        assert_eq!(evaluate(&t), Term::List(vec![Term::int(1), Term::int(4)]));
    }

    #[test]
    fn parse_call_sin() {
        let t = parse_matlab("sin(x)").unwrap();
        assert_eq!(t, Term::app("Sin", vec![Term::symbol("x")]));
    }

    #[test]
    fn parse_power() {
        let t = parse_matlab("x^3").unwrap();
        assert_eq!(t, Term::app("Power", vec![Term::symbol("x"), Term::int(3)]));
    }

    #[test]
    fn parse_root_semicolon_returns_last() {
        let t = parse_matlab("1; 2 + 2").unwrap();
        assert_eq!(evaluate(&t), Term::int(4));
    }

    #[test]
    fn parse_pythagorean() {
        let t = parse_matlab("sin(x)^2 + cos(x)^2").unwrap();
        assert_eq!(evaluate(&Term::app("Simplify", vec![t])), Term::int(1));
    }

    #[test]
    fn parse_diff() {
        let t = parse_matlab("diff(x^3, x)").unwrap();
        let e = evaluate(&t);
        let s = crate::render_wexpr::render_wexpr(&crate::mma_bridge::term_to_wexpr(&e));
        assert!(s.contains('x'), "got {s}");
    }

    #[test]
    fn parse_matrix_array() {
        let t = parse_matlab("[1, 2; 3, 4]").unwrap();
        assert_eq!(
            t,
            Term::List(vec![Term::List(vec![Term::int(1), Term::int(2)]), Term::List(vec![Term::int(3), Term::int(4)]),])
        );
        assert_eq!(crate::render_matlab::render_matlab(&t), "[1, 2; 3, 4]");
    }

    #[test]
    fn parse_integrate_and_sqrt() {
        let t = parse_matlab("int(x^2, x)").unwrap();
        let e = evaluate(&t);
        let s = crate::render_matlab::render_matlab(&e);
        assert!(s.contains('x'), "got {s}");

        let t = parse_matlab("sqrt(9)").unwrap();
        assert_eq!(evaluate(&t), Term::int(3));
    }

    #[test]
    fn parse_comparison() {
        let t = parse_matlab("3 > 2").unwrap();
        assert_eq!(evaluate(&t), Term::int(1));
    }
}
