//! MATLAB dialect surface ↔ Athena [`SemanticOperator`] (Living `27`).
//!
//! Strings here are **dialect display / parse maps only** — never Athena core dispatch.

use athena::{
    Session,
    ir::{ApplicationHead, SemanticOperator, TermNode, UnaryFunction},
    runtime::values::arena::{push_extension, push_semantic},
    types::TermId,
};

/// Map an Athena-facing surface head (often Mathematica-shaped shared IR labels) to a closed op.
pub fn surface_to_semantic(name: &str) -> Option<SemanticOperator> {
    Some(match name {
        "Plus" => SemanticOperator::Add,
        "Times" => SemanticOperator::Multiply,
        "Subtract" => SemanticOperator::Subtract,
        "Divide" => SemanticOperator::Divide,
        "Power" => SemanticOperator::Power,
        "Minus" => SemanticOperator::Negate,
        "DotTimes" => SemanticOperator::ElementwiseMultiply,
        "DotDivide" => SemanticOperator::ElementwiseDivide,
        "DotPower" => SemanticOperator::ElementwisePower,
        "Equal" => SemanticOperator::Equal,
        "Unequal" => SemanticOperator::Unequal,
        "Less" => SemanticOperator::Less,
        "Greater" => SemanticOperator::Greater,
        "LessEqual" => SemanticOperator::LessEqual,
        "GreaterEqual" => SemanticOperator::GreaterEqual,
        "And" => SemanticOperator::And,
        "Or" => SemanticOperator::Or,
        "Not" => SemanticOperator::Not,
        "Range" => SemanticOperator::Range,
        "Simplify" => SemanticOperator::Simplify,
        "Factorial" => SemanticOperator::Factorial,
        "Zeros" => SemanticOperator::Zeros,
        "Ones" => SemanticOperator::Ones,
        "Eye" => SemanticOperator::Eye,
        "Size" => SemanticOperator::Size,
        "Length" => SemanticOperator::Length,
        "Det" | "Determinant" => SemanticOperator::Determinant,
        "Sum" => SemanticOperator::Sum,
        "Sin" => SemanticOperator::from_unary(UnaryFunction::Sin),
        "Cos" => SemanticOperator::from_unary(UnaryFunction::Cos),
        "Tan" => SemanticOperator::from_unary(UnaryFunction::Tan),
        "Exp" => SemanticOperator::from_unary(UnaryFunction::Exp),
        "Log" => SemanticOperator::from_unary(UnaryFunction::Log),
        "Sqrt" => SemanticOperator::from_unary(UnaryFunction::Sqrt),
        "Abs" => SemanticOperator::from_unary(UnaryFunction::Abs),
        _ => return None,
    })
}

/// Map a closed semantic op to the dialect surface label used by MATLAB render / legacy fixtures.
pub fn semantic_to_surface(op: SemanticOperator) -> &'static str {
    match op {
        SemanticOperator::Add => "Plus",
        SemanticOperator::Multiply => "Times",
        SemanticOperator::Negate => "Minus",
        SemanticOperator::ElementwiseMultiply => "DotTimes",
        SemanticOperator::ElementwiseDivide => "DotDivide",
        SemanticOperator::ElementwisePower => "DotPower",
        SemanticOperator::Determinant => "Det",
        SemanticOperator::ApplyHead => "Application",
        SemanticOperator::Unary(f) => f.debug_label(),
        other => other.debug_label(),
    }
}

/// Dialect display head for an application term (Semantic → surface, Extension → registry name).
pub fn application_surface_name(session: &Session, id: TermId) -> Option<String> {
    match session.arena.get(id)? {
        TermNode::Application { head, .. } => match *head {
            ApplicationHead::Semantic(op) => Some(semantic_to_surface(op).to_string()),
            ApplicationHead::Extension(oid) => session.operators.name(oid).map(str::to_string),
        },
        _ => None,
    }
}

/// Push a MATLAB/shared-surface call as Semantic when mapped, else Extension.
pub fn push_matlab_call(session: &mut Session, name: &str, args: Vec<TermId>) -> TermId {
    if let Some(op) = surface_to_semantic(name) {
        push_semantic(session, op, args)
    }
    else {
        let op = session.operators.intern(name);
        push_extension(session, op, args)
    }
}
