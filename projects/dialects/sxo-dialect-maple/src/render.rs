//! Read-only Maple display of Athena result projections.

use athena::{
    Session,
    ir::{ApplicationHead, Atom, SemanticOperator, TermNode},
    numeric::number_to_wire,
    runtime::values::arena::symbol_name,
    types::{CollectionKind, ResultId, TermId, WireNumber},
};

/// A failed display projection, independent of computation status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapleRenderError {
    /// No result exists at this Session-local identifier.
    UnknownResult(ResultId),
    /// A result exists but has no symbolic term projection.
    MissingSymbolicProjection(ResultId),
    /// A projected term is not present in the Session arena.
    UnknownTerm(TermId),
    /// No admitted Maple display contract exists for this construct.
    Unsupported(&'static str),
}

impl std::fmt::Display for MapleRenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownResult(id) => write!(formatter, "unknown Maple result projection: {id:?}"),
            Self::MissingSymbolicProjection(id) => write!(formatter, "Maple result has no symbolic projection: {id:?}"),
            Self::UnknownTerm(id) => write!(formatter, "unknown Maple projected term: {id:?}"),
            Self::Unsupported(construct) => write!(formatter, "unsupported Maple result display: {construct}"),
        }
    }
}

impl std::error::Error for MapleRenderError {}

/// Display a result's symbolic projection without evaluating or allocating terms.
///
/// Text is not a success classification. The caller must expose the same result's
/// status, coverage, conditions and diagnostics independently of this projection.
pub fn render_maple_result_projection(session: &Session, result_id: ResultId) -> Result<String, MapleRenderError> {
    let result = session.results.get(result_id).ok_or(MapleRenderError::UnknownResult(result_id))?;
    let term = result.symbolic_term.ok_or(MapleRenderError::MissingSymbolicProjection(result_id))?;
    render_term(session, term)
}

fn render_term(session: &Session, term: TermId) -> Result<String, MapleRenderError> {
    match session.arena.get(term).ok_or(MapleRenderError::UnknownTerm(term))? {
        TermNode::Atom(Atom::Number(number)) => match number_to_wire(number) {
            WireNumber::Exact(number) => Ok(number.to_render_string()),
            _ => Err(MapleRenderError::Unsupported("non-exact number")),
        },
        TermNode::Atom(Atom::Symbol(_)) => {
            let name = symbol_name(session, term).ok_or(MapleRenderError::Unsupported("unresolved symbol"))?;
            crate::validate_name(&name).map_err(|_| MapleRenderError::Unsupported("non-subset Maple name"))?;
            Ok(name)
        }
        TermNode::Collection { kind: CollectionKind::OrderedCollection, elements } => {
            let elements = elements.iter().map(|child| render_term(session, *child)).collect::<Result<Vec<_>, _>>()?;
            Ok(format!("[{}]", elements.join(", ")))
        }
        TermNode::Application { head: ApplicationHead::Semantic(operator), arguments } => {
            render_arithmetic(session, *operator, arguments)
        }
        TermNode::Collection { .. } => Err(MapleRenderError::Unsupported("non-list collection")),
        TermNode::Application { .. } => Err(MapleRenderError::Unsupported("extension head")),
        TermNode::Atom(_) => Err(MapleRenderError::Unsupported("atom")),
    }
}

fn render_arithmetic(session: &Session, operator: SemanticOperator, arguments: &[TermId]) -> Result<String, MapleRenderError> {
    if operator == SemanticOperator::Negate && arguments.len() == 1 {
        return Ok(format!("-({})", render_term(session, arguments[0])?));
    }
    let token = match (operator, arguments.len()) {
        (SemanticOperator::Add, 2..) => "+",
        (SemanticOperator::Multiply, 2..) => "*",
        (SemanticOperator::Subtract, 2) => "-",
        (SemanticOperator::Divide, 2) => "/",
        (SemanticOperator::Power, 2) => "^",
        _ => return Err(MapleRenderError::Unsupported("operator or arity")),
    };
    let operands = arguments
        .iter()
        .map(|child| render_term(session, *child).map(|text| format!("({text})")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("({})", operands.join(&format!(" {token} "))))
}
