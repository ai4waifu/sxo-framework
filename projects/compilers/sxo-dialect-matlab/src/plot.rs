//! `plot(f,x,a,b)` lowering → Athena sampling → Apollo SVG.

use athena::{
    plot::SampleDomain,
    plot::SamplingPolicy,
    Session,
    types::TermId,
    runtime::values::arena::application_arguments,
    runtime::values::arena::application_head_name,
    runtime::values::arena::number_from_id,
    numeric::to_f64_lossy,
    runtime::values::arena::symbol_name,
};
use sxo_adapter_apollo::{AdapterError, plot_1d_svg};
use sxo_types::SxoError;

const DEFAULT_SAMPLES: u32 = 128;

/// Try to render MATLAB-style `plot(f,x,a,b)` as SVG.
pub fn try_plot_svg(session: &mut Session, id: TermId) -> Option<Result<String, SxoError>> {
    let spec = extract_plot_1d(session, id)?;
    Some(render_plot_1d(session, spec))
}

struct Plot1dSpec {
    expr: TermId,
    var: String,
    start: f64,
    end: f64,
}

fn render_plot_1d(session: &mut Session, spec: Plot1dSpec) -> Result<String, SxoError> {
    plot_1d_svg(
        session,
        spec.expr,
        &spec.var,
        SampleDomain::new(spec.start, spec.end),
        SamplingPolicy::samples(DEFAULT_SAMPLES),
    )
    .map_err(adapter_to_sxo)
}

fn adapter_to_sxo(err: AdapterError) -> SxoError {
    match err {
        AdapterError::Athena(d) => SxoError::from_diagnostic(d),
        AdapterError::Apollo { code } => SxoError::new(code),
    }
}

fn extract_plot_1d(session: &mut Session, id: TermId) -> Option<Plot1dSpec> {
    if application_head_name(session, id).as_deref() != Some("plot") {
        return None;
    }
    let args = application_arguments(session, id)?;
    if args.len() != 4 {
        return None;
    }
    let var = symbol_name(session, args[1])?;
    let start = term_as_f64(session, args[2])?;
    let end = term_as_f64(session, args[3])?;
    Some(Plot1dSpec { expr: args[0], var, start, end })
}

fn term_as_f64(session: &mut Session, id: TermId) -> Option<f64> {
    if let Some(n) = number_from_id(session, id) {
        return to_f64_lossy(n);
    }
    let folded = session.evaluate(id);
    to_f64_lossy(number_from_id(session, folded)?)
}
