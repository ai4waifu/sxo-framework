//! Lower Mathematica Form ([`WExpr`]) to engine terms and back.

use athena::{Atom, Term, clone_number};

use crate::form::{WAtom, WExpr};

/// Structural `WExpr` → engine `Term`.
pub fn wexpr_to_term(w: &WExpr) -> Term {
    match w {
        WExpr::Atom(a) => Term::Atom(match a {
            WAtom::Number(n) => Atom::Number(clone_number(n)),
            WAtom::String(s) => Atom::String(s.clone()),
            WAtom::Symbol(s) if s == "True" => Atom::Boolean(true),
            WAtom::Symbol(s) if s == "False" => Atom::Boolean(false),
            WAtom::Symbol(s) if s == "Null" => Atom::Null,
            WAtom::Symbol(s) => Atom::Symbol(s.clone()),
        }),
        WExpr::List(items) => Term::List(items.iter().map(wexpr_to_term).collect()),
        WExpr::Call { head, args } => {
            Term::Application { head: Box::new(wexpr_to_term(head)), arguments: args.iter().map(wexpr_to_term).collect() }
        }
    }
}

/// Engine `Term` → structural `WExpr`.
pub fn term_to_wexpr(t: &Term) -> WExpr {
    match t {
        Term::Atom(a) => WExpr::Atom(match a {
            Atom::Number(n) => WAtom::Number(clone_number(n)),
            Atom::String(s) => WAtom::String(s.clone()),
            Atom::Boolean(true) => WAtom::Symbol("True".into()),
            Atom::Boolean(false) => WAtom::Symbol("False".into()),
            Atom::Null => WAtom::Symbol("Null".into()),
            Atom::Symbol(s) => WAtom::Symbol(s.clone()),
        }),
        Term::List(items) => WExpr::List(items.iter().map(term_to_wexpr).collect()),
        Term::Application { head, arguments: args } => {
            WExpr::Call { head: Box::new(term_to_wexpr(head)), args: args.iter().map(term_to_wexpr).collect() }
        }
    }
}
