//! Oak-owned Maple syntax to independent SXO forms.

use oak_core::{Builder, ParseSession, source::SourceText};
use oak_maple::{
    MapleBuilder, MapleLanguage,
    ast::{MapleExpression, MapleExpressionKind, MapleOperator as OakOperator, MapleRoot},
};

use crate::{MapleError, MapleForm, MapleOperator, MapleProgram, MapleSpan, MapleSyntaxDiagnostic};

/// Parse source without trimming it or allocating Athena terms.
///
/// Empty syntax roots are retained; execution rejects them explicitly.
pub fn parse_maple_program(input: &str) -> Result<MapleProgram, MapleError> {
    let language = MapleLanguage::default();
    let source = SourceText::new(input);
    let mut cache = ParseSession::<MapleLanguage>::default();
    let output = MapleBuilder::new(&language).build(&source, &[], &mut cache);
    if !output.diagnostics.is_empty() {
        return Err(MapleError::Syntax(
            output
                .diagnostics
                .iter()
                .map(|error| MapleSyntaxDiagnostic { message: error.to_string(), offset: error.source_offset() })
                .collect(),
        ));
    }
    let root = output.result.map_err(|error| {
        MapleError::Syntax(vec![MapleSyntaxDiagnostic { message: error.to_string(), offset: error.source_offset() }])
    })?;
    program_from_ast(&root)
}

/// Parse exactly one expression, preserving source locations and spelling.
pub fn parse_maple_form(input: &str) -> Result<MapleForm, MapleError> {
    let mut program = parse_maple_program(input)?;
    match program.expressions.len() {
        0 => Err(MapleError::EmptyProgram),
        1 => Ok(program.expressions.remove(0)),
        _ => Err(MapleError::Syntax(vec![MapleSyntaxDiagnostic {
            message: "expected one Maple expression, found a source sequence".to_owned(),
            offset: program.expressions.get(1).and_then(|form| match form {
                MapleForm::Located { span, .. } => Some(span.start),
                _ => None,
            }),
        }])),
    }
}

/// Convert owned Oak syntax without visiting CST, rendering, or evaluating.
pub fn program_from_ast(root: &MapleRoot) -> Result<MapleProgram, MapleError> {
    let span = MapleSpan { start: root.span.start, end: root.span.end };
    span.to_source_span()?;
    Ok(MapleProgram { expressions: root.expressions.iter().map(form_from_ast).collect::<Result<_, _>>()?, span: Some(span) })
}

/// Convert a language-specific expression while preserving each source extent.
pub fn form_from_ast(expression: &MapleExpression) -> Result<MapleForm, MapleError> {
    let span = MapleSpan { start: expression.span.start, end: expression.span.end };
    span.to_source_span()?;
    let form = match &expression.kind {
        MapleExpressionKind::Symbol(name) => MapleForm::Symbol(name.clone()),
        MapleExpressionKind::Integer(text) => MapleForm::Integer(text.clone()),
        MapleExpressionKind::Decimal(text) => MapleForm::Decimal(text.clone()),
        MapleExpressionKind::List(items) => MapleForm::List(items.iter().map(form_from_ast).collect::<Result<_, _>>()?),
        MapleExpressionKind::Call { name, arguments } => {
            MapleForm::Call { name: name.clone(), arguments: arguments.iter().map(form_from_ast).collect::<Result<_, _>>()? }
        }
        MapleExpressionKind::Grouped(inner) => MapleForm::Grouped(Box::new(form_from_ast(inner)?)),
        MapleExpressionKind::Unary { operator, operand } => {
            let operand = Box::new(form_from_ast(operand)?);
            match operator {
                OakOperator::Add => MapleForm::Positive(operand),
                OakOperator::Subtract => MapleForm::Negative(operand),
                _ => {
                    return Err(MapleError::Syntax(vec![MapleSyntaxDiagnostic {
                        message: "invalid Maple unary operator in typed AST".to_owned(),
                        offset: Some(span.start),
                    }]));
                }
            }
        }
        MapleExpressionKind::Binary { operator, left, right } => MapleForm::Binary {
            operator: match operator {
                OakOperator::Add => MapleOperator::Add,
                OakOperator::Subtract => MapleOperator::Subtract,
                OakOperator::Multiply => MapleOperator::Multiply,
                OakOperator::Divide => MapleOperator::Divide,
                OakOperator::Power => MapleOperator::Power,
            },
            left: Box::new(form_from_ast(left)?),
            right: Box::new(form_from_ast(right)?),
        },
    };
    Ok(MapleForm::Located { span, expression: Box::new(form) })
}

#[cfg(test)]
#[path = "parse_tests.rs"]
mod tests;
