//! `Plot[f,{x,a,b}]` lowering → Athena sampling → Apollo SVG.

use athena::{Atom, SampleDomain, SamplingPolicy, Term, clone_term, evaluate, number_from_term, numeric::to_f64_lossy};
use sxo_adapter_apollo::{AdapterError, plot_1d_svg};
use sxo_types::SxoError;

const DEFAULT_SAMPLES: u32 = 128;

/// Try to render Mathematica `Plot[f,{x,a,b}]` as SVG.
pub fn try_plot_svg(term: &Term) -> Option<Result<String, SxoError>> {
    extract_plot_1d(term).map(render_plot_1d)
}

struct Plot1dSpec {
    expr: Term,
    var: String,
    start: f64,
    end: f64,
}

fn render_plot_1d(spec: Plot1dSpec) -> Result<String, SxoError> {
    plot_1d_svg(&spec.expr, &spec.var, SampleDomain::new(spec.start, spec.end), SamplingPolicy::samples(DEFAULT_SAMPLES))
        .map_err(adapter_to_sxo)
}

fn adapter_to_sxo(err: AdapterError) -> SxoError {
    match err {
        AdapterError::Athena(d) => SxoError::from_diagnostic(d),
        AdapterError::Apollo { code } => SxoError::new(code),
    }
}

fn extract_plot_1d(term: &Term) -> Option<Plot1dSpec> {
    let Term::Application { head, arguments: args } = term
    else {
        return None;
    };
    let name = match head.as_ref() {
        Term::Atom(Atom::Symbol(s)) => s.as_str(),
        _ => return None,
    };
    if name != "Plot" || args.len() != 2 {
        return None;
    }
    extract_mma_plot(&args[0], &args[1])
}

fn extract_mma_plot(expr: &Term, iterator: &Term) -> Option<Plot1dSpec> {
    let Term::Application { head, arguments: parts } = iterator
    else {
        if let Term::List(parts) = iterator {
            return list_iterator(expr, parts);
        }
        return None;
    };
    if head.is_symbol("List") && parts.len() == 3 {
        return list_iterator(expr, parts);
    }
    None
}

fn list_iterator(expr: &Term, parts: &[Term]) -> Option<Plot1dSpec> {
    let var = match &parts[0] {
        Term::Atom(Atom::Symbol(s)) => s.clone(),
        _ => return None,
    };
    let start = term_as_f64(&parts[1])?;
    let end = term_as_f64(&parts[2])?;
    Some(Plot1dSpec { expr: clone_term(expr), var, start, end })
}

/// Literal numbers and unary-minus forms (`Times[-1, n]`) after light eval.
fn term_as_f64(term: &Term) -> Option<f64> {
    if let Some(n) = number_from_term(term) {
        return to_f64_lossy(n);
    }
    let folded = evaluate(term);
    to_f64_lossy(number_from_term(&folded)?)
}
