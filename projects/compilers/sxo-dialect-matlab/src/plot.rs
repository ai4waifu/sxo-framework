//! `plot(f,x,a,b)` lowering → Athena sampling → Apollo SVG.

use athena::{Atom, SampleDomain, SamplingPolicy, Term, number_from_term, numeric::to_f64_lossy};
use sxo_adapter_apollo::{AdapterError, plot_1d_svg};
use sxo_types::SxoError;

const DEFAULT_SAMPLES: u32 = 128;

/// Try to render MATLAB-style `plot(f,x,a,b)` as SVG.
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
    if name != "plot" || args.len() != 4 {
        return None;
    }
    let var = match &args[1] {
        Term::Atom(Atom::Symbol(s)) => s.clone(),
        _ => return None,
    };
    let start = to_f64_lossy(number_from_term(&args[2])?)?;
    let end = to_f64_lossy(number_from_term(&args[3])?)?;
    Some(Plot1dSpec { expr: args[0].clone(), var, start, end })
}
