//! Independent Maple surface objects and neutral Athena requests.

pub mod parse;
mod render;

pub use render::{MapleRenderError, render_maple_result_projection};

use science::{
    Session,
    api::{CasRequest, ControlPlan},
    ir::{SemanticOperator, TermBuilder},
    numeric::number_from_wire,
    types::{SourceSpan, TermId, WireNumber},
};

/// Owned UTF-8 byte range of a Maple source construct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapleSpan {
    /// Inclusive starting byte offset.
    pub start: usize,
    /// Exclusive ending byte offset.
    pub end: usize,
}

impl MapleSpan {
    pub(crate) fn to_source_span(self) -> Result<SourceSpan, MapleError> {
        if self.start > self.end {
            return Err(MapleError::InvalidSpan(self));
        }
        Ok(SourceSpan {
            start: u32::try_from(self.start).map_err(|_| MapleError::InvalidSpan(self))?,
            end: u32::try_from(self.end).map_err(|_| MapleError::InvalidSpan(self))?,
        })
    }
}

/// A source program retains expression order independently of an Athena session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapleProgram {
    /// Ordered expressions separated by source semicolons.
    pub expressions: Vec<MapleForm>,
    /// Optional extent of the complete source input.
    pub span: Option<MapleSpan>,
}

/// An owned Maple expression, without session-bound mathematical identifiers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapleForm {
    /// Source provenance, independent of session-owned terms.
    Located {
        /// Original UTF-8 byte range.
        span: MapleSpan,
        /// Owned expression at that range.
        expression: Box<MapleForm>,
    },
    /// Exact unsigned integer spelling. Signs use unary operators.
    Integer(String),
    /// Decimal spelling, retained but not yet admitted for execution.
    Decimal(String),
    /// A Maple name.
    Symbol(String),
    /// A list, never implicitly a matrix.
    List(Vec<MapleForm>),
    /// A source-level named call, currently unsupported for execution.
    Call {
        /// Source-level function name.
        name: String,
        /// Ordered source arguments.
        arguments: Vec<MapleForm>,
    },
    /// Explicit source parentheses.
    Grouped(Box<MapleForm>),
    /// Unary plus.
    Positive(Box<MapleForm>),
    /// Unary minus.
    Negative(Box<MapleForm>),
    /// A typed binary arithmetic expression.
    Binary {
        /// Arithmetic operator.
        operator: MapleOperator,
        /// Left operand.
        left: Box<MapleForm>,
        /// Right operand.
        right: Box<MapleForm>,
    },
}

/// Maple binary operators supported by the scalar lowering boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapleOperator {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division.
    Divide,
    /// Explicitly grouped exponentiation.
    Power,
}

/// An owned Oak syntax diagnostic, retaining its optional byte offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapleSyntaxDiagnostic {
    /// Source parser message, never an Athena diagnostic code.
    pub message: String,
    /// UTF-8 byte offset when provided by Oak.
    pub offset: Option<usize>,
}

/// Frontend failure category, not an Athena diagnostic code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapleError {
    /// Source syntax errors. Invalid syntax never publishes executable Form.
    Syntax(Vec<MapleSyntaxDiagnostic>),
    /// Empty syntax roots have no executable expression.
    EmptyProgram,
    /// Reversed or unrepresentable source range.
    InvalidSpan(MapleSpan),
    /// A frontend failure associated with its nearest source construct.
    Located {
        /// Source construct range.
        span: MapleSpan,
        /// Original typed frontend failure.
        error: Box<MapleError>,
    },
    /// Invalid unsigned integer spelling.
    InvalidInteger(String),
    /// Invalid or reserved Maple name.
    InvalidName(String),
    /// A surface construct without an admitted lowering contract.
    Unsupported(String),
}

impl std::fmt::Display for MapleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(diagnostics) => {
                formatter.write_str("invalid Maple syntax")?;
                for diagnostic in diagnostics {
                    write!(formatter, ": {}", diagnostic.message)?;
                    if let Some(offset) = diagnostic.offset {
                        write!(formatter, " at byte {offset}")?;
                    }
                }
                Ok(())
            }
            Self::EmptyProgram => formatter.write_str("Maple program has no executable expression"),
            Self::InvalidSpan(span) => write!(formatter, "invalid Maple byte range: {}..{}", span.start, span.end),
            Self::Located { span, error } => write!(formatter, "{error} at bytes {}..{}", span.start, span.end),
            Self::InvalidInteger(text) => write!(formatter, "invalid Maple integer: {text}"),
            Self::InvalidName(name) => write!(formatter, "invalid Maple name: {name}"),
            Self::Unsupported(construct) => write!(formatter, "unsupported Maple construct: {construct}"),
        }
    }
}

impl std::error::Error for MapleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Located { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}

pub(crate) fn validate_name(name: &str) -> Result<(), MapleError> {
    let mut bytes = name.bytes();
    let valid_start = bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    let valid_rest = bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    let reserved = "and assuming break by catch description do done elif else end error export fi finally for from global if implies in intersect local minus mod module next not od option options or proc quit read return save stop subset then to try union until use uses while xor"
        .split_whitespace().any(|keyword| keyword == name);
    if valid_start && valid_rest && !reserved { Ok(()) } else { Err(MapleError::InvalidName(name.to_owned())) }
}

fn integer_wire(text: &str) -> Result<WireNumber, MapleError> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(MapleError::InvalidInteger(text.to_owned()));
    }
    WireNumber::from_decimal_str(text).ok_or_else(|| MapleError::InvalidInteger(text.to_owned()))
}

fn validate_lowering(form: &MapleForm) -> Result<(), MapleError> {
    match form {
        MapleForm::Located { span, expression } => {
            span.to_source_span()?;
            validate_lowering(expression).map_err(|error| match error {
                located @ MapleError::Located { .. } => located,
                error => MapleError::Located { span: *span, error: Box::new(error) },
            })
        }
        MapleForm::Integer(text) => {
            let wire = integer_wire(text)?;
            number_from_wire(&wire).map_err(|_| MapleError::InvalidInteger(text.clone()))?;
            Ok(())
        }
        MapleForm::Symbol(name) => validate_name(name),
        MapleForm::Decimal(_) => Err(MapleError::Unsupported("decimal precision policy".to_owned())),
        MapleForm::Call { name, .. } => Err(MapleError::Unsupported(format!("call {name}"))),
        MapleForm::List(items) => items.iter().try_for_each(validate_lowering),
        MapleForm::Grouped(inner) | MapleForm::Positive(inner) | MapleForm::Negative(inner) => validate_lowering(inner),
        MapleForm::Binary { left, right, .. } => {
            validate_lowering(left)?;
            validate_lowering(right)
        }
    }
}

fn materialize(session: &mut Session, form: &MapleForm, span: SourceSpan) -> Result<TermId, MapleError> {
    match form {
        MapleForm::Located { span, expression } => materialize(session, expression, span.to_source_span()?),
        MapleForm::Integer(text) => {
            let number = number_from_wire(&integer_wire(text)?).map_err(|_| MapleError::InvalidInteger(text.clone()))?;
            Ok(TermBuilder::new(&mut session.arena).number(number, span))
        }
        MapleForm::Symbol(name) => Ok(TermBuilder::new(&mut session.arena).symbol(name, span)),
        MapleForm::List(items) => {
            let terms = items.iter().map(|item| materialize(session, item, span)).collect::<Result<Vec<_>, _>>()?;
            Ok(TermBuilder::new(&mut session.arena).list(terms, span))
        }
        MapleForm::Grouped(inner) | MapleForm::Positive(inner) => materialize(session, inner, span),
        MapleForm::Negative(inner) => {
            let term = materialize(session, inner, span)?;
            Ok(TermBuilder::new(&mut session.arena).application_semantic(SemanticOperator::Negate, vec![term], span))
        }
        MapleForm::Binary { operator, left, right } => {
            let left = materialize(session, left, span)?;
            let right = materialize(session, right, span)?;
            let semantic = match operator {
                MapleOperator::Add => SemanticOperator::Add,
                MapleOperator::Subtract => SemanticOperator::Subtract,
                MapleOperator::Multiply => SemanticOperator::Multiply,
                MapleOperator::Divide => SemanticOperator::Divide,
                MapleOperator::Power => SemanticOperator::Power,
            };
            Ok(TermBuilder::new(&mut session.arena).application_semantic(semantic, vec![left, right], span))
        }
        MapleForm::Decimal(_) => Err(MapleError::Unsupported("decimal precision policy".to_owned())),
        MapleForm::Call { name, .. } => Err(MapleError::Unsupported(format!("call {name}"))),
    }
}

/// Validate the entire form before allocating terms, then lower once into Athena.
///
/// This does not evaluate, render, reparse, or introduce frontend mathematical storage.
pub fn lower_request(session: &mut Session, form: &MapleForm) -> Result<CasRequest, MapleError> {
    validate_lowering(form)?;
    materialize(session, form, SourceSpan::default()).map(CasRequest::Term)
}

/// Validate every expression before allocation and lower the source sequence.
///
/// A single expression remains a term request. Multiple expressions use Athena
/// control flow, whose observed result is the last expression. This is not a
/// transcript of the outputs that an interactive Maple console would print.
pub fn lower_program_request(session: &mut Session, program: &MapleProgram) -> Result<CasRequest, MapleError> {
    let span = program.span.map(MapleSpan::to_source_span).transpose()?.unwrap_or_default();
    if program.expressions.is_empty() {
        return Err(MapleError::EmptyProgram);
    }
    program.expressions.iter().try_for_each(validate_lowering)?;
    let mut steps = program
        .expressions
        .iter()
        .map(|form| materialize(session, form, span).map(CasRequest::Term))
        .collect::<Result<Vec<_>, _>>()?;
    if steps.len() == 1 { Ok(steps.remove(0)) } else { Ok(CasRequest::Control(ControlPlan::Sequence { steps })) }
}

/// Render an owned source form with explicit arithmetic grouping.
///
/// Rendering is display-only and is never used by lowering.
pub fn render_maple_form(form: &MapleForm) -> String {
    match form {
        MapleForm::Located { expression, .. } => render_maple_form(expression),
        MapleForm::Integer(text) | MapleForm::Decimal(text) | MapleForm::Symbol(text) => text.clone(),
        MapleForm::List(items) => format!("[{}]", items.iter().map(render_maple_form).collect::<Vec<_>>().join(", ")),
        MapleForm::Call { name, arguments } => {
            format!("{}({})", name, arguments.iter().map(render_maple_form).collect::<Vec<_>>().join(", "))
        }
        MapleForm::Grouped(inner) => format!("({})", render_maple_form(inner)),
        MapleForm::Positive(inner) => format!("+({})", render_maple_form(inner)),
        MapleForm::Negative(inner) => format!("-({})", render_maple_form(inner)),
        MapleForm::Binary { operator, left, right } => {
            let token = match operator {
                MapleOperator::Add => "+",
                MapleOperator::Subtract => "-",
                MapleOperator::Multiply => "*",
                MapleOperator::Divide => "/",
                MapleOperator::Power => "^",
            };
            format!("({} {} {})", render_maple_form(left), token, render_maple_form(right))
        }
    }
}
