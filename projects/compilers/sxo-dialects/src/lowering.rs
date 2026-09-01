//! Lower Athena [`Term`] into kernel IR ([`TermArena`](athena::TermArena)).

use std::collections::HashMap;

use athena::{Atom, AtomKind, OperatorId, SourceSpan, Term, TermArena, TermId, TermKind};
use sxo_types::SxoError;

/// Athena kernel IR bundle after lowering.
#[derive(Debug)]
pub struct KernelTerm {
    /// Owning arena.
    pub arena: TermArena,
    /// Root term id.
    pub root: TermId,
}

/// Map Athena tree into kernel IR.
pub fn lower_to_kernel(term: &Term) -> Result<KernelTerm, SxoError> {
    let mut arena = TermArena::new();
    let mut ops = OperatorRegistry::default();
    let root = lower_term(&mut arena, &mut ops, term, SourceSpan::default())?;
    arena.verify(root).map_err(SxoError::from_diagnostic)?;
    Ok(KernelTerm { arena, root })
}

fn lower_term(arena: &mut TermArena, ops: &mut OperatorRegistry, term: &Term, span: SourceSpan) -> Result<TermId, SxoError> {
    match term {
        Term::Atom(Atom::Number(n)) => Ok(arena.push(TermKind::Atom(AtomKind::Number(n.clone())), span)),
        Term::Atom(Atom::String(s)) => Ok(arena.push(TermKind::Atom(AtomKind::String(s.clone())), span)),
        Term::Atom(Atom::Symbol(s)) => {
            let sym = arena.symbols_mut().intern(s.clone());
            Ok(arena.push(TermKind::Atom(AtomKind::Symbol(sym)), span))
        }
        Term::List(items) => {
            let mut ids = Vec::with_capacity(items.len());
            for item in items {
                ids.push(lower_term(arena, ops, item, span)?);
            }
            Ok(arena.push(TermKind::List(ids), span))
        }
        Term::Application { head, arguments: args } => {
            let head_name = head.head_name().ok_or_else(|| SxoError::new("lowering: application head must be a symbol"))?;
            let op = ops.intern(head_name);
            let mut arg_ids = Vec::with_capacity(args.len());
            for arg in args {
                arg_ids.push(lower_term(arena, ops, arg, span)?);
            }
            Ok(arena.push(TermKind::App { op, args: arg_ids }, span))
        }
    }
}

#[derive(Debug, Default)]
struct OperatorRegistry {
    ids: HashMap<String, OperatorId>,
    next: u32,
}

impl OperatorRegistry {
    fn intern(&mut self, name: &str) -> OperatorId {
        if let Some(id) = self.ids.get(name) {
            return *id;
        }
        let id = OperatorId(self.next);
        self.next += 1;
        self.ids.insert(name.to_string(), id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_plus_app() {
        let t = Term::app("Plus", vec![Term::int(1), Term::int(2)]);
        let k = lower_to_kernel(&t).unwrap();
        assert!(matches!(k.arena.get(k.root), Some(TermKind::App { .. })));
    }
}
