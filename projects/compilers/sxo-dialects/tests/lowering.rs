//! Integration tests for lowering.

use sxo_dialects::{Term, TermKind, lower_to_kernel};

#[test]
fn lower_plus_app() {
    let t = Term::apply("Plus", vec![Term::int(1), Term::int(2)]);
    let k = lower_to_kernel(&t).unwrap();
    assert!(matches!(k.arena.get(k.root), Some(TermKind::App { .. })));
}
