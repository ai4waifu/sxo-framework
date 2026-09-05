//! `Plot[f,{x,a,b}]` lowering → Athena sampling → Apollo SVG.

use athena::{
    ir::TermNode,
    numeric::to_f64_lossy,
    plot::SampleDomain,
    plot::SamplingPolicy,
    runtime::values::arena::{application_arguments, application_display_name, number_from_id, symbol_name},
    Session,
    types::TermId,
};
use sxo_adapter_apollo::{AdapterError, plot_1d_svg};
use sxo_types::SxoError;

const DEFAULT_SAMPLES: u32 = 128;

/// Try to render Mathematica `Plot[f,{x,a,b}]` as SVG.
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
    if application_display_name(session, id).as_deref() != Some("Plot") {
        return None;
    }
    let args = application_arguments(session, id)?;
    if args.len() != 2 {
        return None;
    }
    extract_mma_plot(session, args[0], args[1])
}

fn extract_mma_plot(session: &mut Session, expr: TermId, iterator: TermId) -> Option<Plot1dSpec> {
    if let Some(TermNode::Collection { elements: parts, .. }) = session.arena.get(iterator) {
        if parts.len() == 3 {
            let parts = parts.clone();
            return list_iterator(session, expr, &parts);
        }
    }
    if application_display_name(session, iterator).as_deref() == Some("List") {
        let parts = application_arguments(session, iterator)?;
        if parts.len() == 3 {
            return list_iterator(session, expr, &parts);
        }
    }
    None
}

fn list_iterator(session: &mut Session, expr: TermId, parts: &[TermId]) -> Option<Plot1dSpec> {
    let var = symbol_name(session, parts[0])?;
    let start = term_as_f64(session, parts[1])?;
    let end = term_as_f64(session, parts[2])?;
    Some(Plot1dSpec { expr, var, start, end })
}

/// Literal numbers and unary-minus forms after light eval.
fn term_as_f64(session: &mut Session, id: TermId) -> Option<f64> {
    if let Some(n) = number_from_id(session, id) {
        return to_f64_lossy(n);
    }
    let folded = session.evaluate(id);
    to_f64_lossy(number_from_id(session, folded)?)
}
