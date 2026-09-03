//! Host integration tests across dialect crates and Athena.

use athena::{
    Atom, AtomKind, CalculusRequest, DomainRequest, OperatorId, SourceSpan, Term, TermArena, TermKind, clone_number,
    try_calculus_request,
};
use sxo_dialect_mathematica::{WExpr, parse_number_literal, term_to_wexpr, wexpr_to_term};
use sxo_napi::session::Session;
use sxo_types::SxoError;

fn lower_to_kernel(term: &Term) -> Result<(TermArena, athena::TermId), SxoError> {
    use std::collections::HashMap;

    let mut arena = TermArena::new();
    let mut ops: HashMap<String, OperatorId> = HashMap::new();
    let mut next = 0u32;

    fn lower(
        arena: &mut TermArena,
        ops: &mut HashMap<String, OperatorId>,
        next: &mut u32,
        term: &Term,
        span: SourceSpan,
    ) -> Result<athena::TermId, SxoError> {
        match term {
            Term::Atom(Atom::Number(n)) => Ok(arena.push(TermKind::Atom(AtomKind::Number(clone_number(n))), span)),
            Term::Atom(Atom::String(s)) => Ok(arena.push(TermKind::Atom(AtomKind::String(s.clone())), span)),
            Term::Atom(Atom::Boolean(b)) => Ok(arena.push(TermKind::Atom(AtomKind::Boolean(*b)), span)),
            Term::Atom(Atom::Null) => Ok(arena.push(TermKind::Atom(AtomKind::Null), span)),
            Term::Atom(Atom::Symbol(s)) => {
                let sym = arena.symbols_mut().intern(s.clone());
                Ok(arena.push(TermKind::Atom(AtomKind::Symbol(sym)), span))
            }
            Term::List(items) => {
                let mut ids = Vec::with_capacity(items.len());
                for item in items {
                    ids.push(lower(arena, ops, next, item, span)?);
                }
                Ok(arena.push(TermKind::List(ids), span))
            }
            Term::Application { head, arguments: args } => {
                let head_name = head.head_name().ok_or_else(|| SxoError::new("lowering: application head must be a symbol"))?;
                let op = if let Some(id) = ops.get(head_name) {
                    *id
                }
                else {
                    let id = OperatorId(*next);
                    *next += 1;
                    ops.insert(head_name.to_string(), id);
                    id
                };
                let mut arg_ids = Vec::with_capacity(args.len());
                for arg in args {
                    arg_ids.push(lower(arena, ops, next, arg, span)?);
                }
                Ok(arena.push(TermKind::App { op, args: arg_ids }, span))
            }
        }
    }

    let root = lower(&mut arena, &mut ops, &mut next, term, SourceSpan::default())?;
    arena.verify(root).map_err(SxoError::from_diagnostic)?;
    Ok((arena, root))
}

#[test]
fn math_evaluate_arith() {
    let session = Session::new();
    let e = session.evaluate_mathematica("1 + 2 * 3").unwrap();
    assert_eq!(e, Term::int(7));
}

#[test]
fn wexpr_is_not_term() {
    let w = WExpr::call("Sin", vec![WExpr::symbol("x")]);
    let t = wexpr_to_term(&w);
    assert_eq!(t, Term::apply("Sin", vec![Term::symbol("x")]));
    assert_eq!(term_to_wexpr(&t), w);
}

#[test]
fn big_integer_arithmetic() {
    let session = Session::new();
    let e = session.evaluate_mathematica("99999999999999999999 + 1").unwrap();
    let expected = Term::number(parse_number_literal("100000000000000000000").unwrap());
    assert_eq!(e, expected);
}

#[test]
fn bridge_lowers_to_kernel() {
    let session = Session::new();
    let w = session.parse_mathematica("1 + 2").unwrap();
    let t = session.from_mathematica(&w);
    let (arena, root) = lower_to_kernel(&t).unwrap();
    assert!(matches!(arena.get(root), Some(TermKind::App { .. })));
}

#[test]
fn dialect_d_limit_series_lower_to_domain() {
    let session = Session::new();

    let d_term = session.from_mathematica(&session.parse_mathematica("D[x^3, x]").unwrap());
    assert!(matches!(
        try_calculus_request(&d_term).map(DomainRequest::Calculus),
        Some(DomainRequest::Calculus(CalculusRequest::Derivative { .. }))
    ));
    let d_out = session.evaluate(&d_term);
    let d_s = session.render_as_wolfram(&d_out);
    assert!(d_s.contains('x'), "got {d_s}");
}

#[test]
fn session_set_persists_across_mathematica_evaluates() {
    let session = Session::new();
    assert_eq!(session.evaluate_mathematica("x = 5").unwrap(), Term::int(5));
    assert_eq!(session.evaluate_mathematica("x + 1").unwrap(), Term::int(6));
    session.clear_definitions();
    let cleared = session.evaluate_mathematica("x + 1").unwrap();
    assert!(
        matches!(&cleared, Term::Application { head, .. } if head.is_symbol("Plus")),
        "expected free Plus after clear, got {cleared:?}"
    );
}

#[test]
fn session_set_persists_across_matlab_evaluates() {
    let session = Session::new();
    assert_eq!(session.evaluate_matlab("x = 5").unwrap(), Term::int(5));
    assert_eq!(session.evaluate_matlab("x + 1").unwrap(), Term::int(6));
}
