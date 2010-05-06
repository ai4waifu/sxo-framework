//! Render [`GpForm`] as GP-ish text (structural only).

use crate::form::{GpAtom, GpForm};

/// Render a Form tree as PARI/GP-like source (no kernel roundtrip).
pub fn render_gp(form: &GpForm) -> String {
    match form {
        GpForm::Atom(GpAtom::NumberLiteral(t)) => t.clone(),
        GpForm::Atom(GpAtom::String(s)) => format!("\"{s}\""),
        GpForm::Atom(GpAtom::Symbol(s)) => s.clone(),
        GpForm::Atom(GpAtom::Bool(true)) => "1".into(),
        GpForm::Atom(GpAtom::Bool(false)) => "0".into(),
        GpForm::List(items) => {
            let body = items.iter().map(render_gp).collect::<Vec<_>>().join(", ");
            format!("[{body}]")
        }
        GpForm::Call { head, args } => {
            let body = args.iter().map(render_gp).collect::<Vec<_>>().join(", ");
            format!("{head}({body})")
        }
    }
}
