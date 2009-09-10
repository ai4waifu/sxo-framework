//! `Plot` / `plot` 薄 lowering → Athena 采样 → Apollo SVG。

use athena::{Atom, SampleDomain, SamplingPolicy, Term, number_from_term};
use sxo_adapter_apollo::{AdapterError, plot_1d_svg};
use sxo_types::SxoError;

/// 默认采样点数。
const DEFAULT_SAMPLES: u32 = 128;

/// 尝试将 `Plot[f,{x,a,b}]` 或 `plot(f,x,a,b)` 形态渲染为 SVG。
pub fn try_plot_svg(term: &Term) -> Option<Result<String, SxoError>> {
    match extract_plot_1d(term) {
        Some(spec) => Some(render_plot_1d(spec)),
        None => None,
    }
}

struct Plot1dSpec {
    expr: Term,
    var: String,
    start: f64,
    end: f64,
}

fn render_plot_1d(spec: Plot1dSpec) -> Result<String, SxoError> {
    plot_1d_svg(
        &spec.expr,
        &spec.var,
        SampleDomain::new(spec.start, spec.end),
        SamplingPolicy {
            max_samples: DEFAULT_SAMPLES,
        },
    )
    .map_err(adapter_to_sxo)
}

fn adapter_to_sxo(err: AdapterError) -> SxoError {
    match err {
        AdapterError::Athena(d) => SxoError::from_diagnostic(d),
        AdapterError::Apollo { code } => SxoError::new(code),
    }
}

/// 识别 Mathematica `Plot[f,{x,a,b}]` 与 MATLAB 风格 `plot(f,x,a,b)`（四元应用）。
fn extract_plot_1d(term: &Term) -> Option<Plot1dSpec> {
    let Term::Application { head, arguments: args } = term else {
        return None;
    };
    let name = match head.as_ref() {
        Term::Atom(Atom::Symbol(s)) => s.as_str(),
        _ => return None,
    };

    match name {
        "Plot" if args.len() == 2 => extract_mma_plot(&args[0], &args[1]),
        "plot" if args.len() == 4 => extract_matlab_plot(&args[0], &args[1], &args[2], &args[3]),
        _ => None,
    }
}

fn extract_mma_plot(expr: &Term, iterator: &Term) -> Option<Plot1dSpec> {
    // {x, a, b} as List[x, a, b]
    let Term::Application { head, arguments: parts } = iterator else {
        // Also accept Term::List
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
    let start = number_from_term(&parts[1])?.to_f64_lossy()?;
    let end = number_from_term(&parts[2])?.to_f64_lossy()?;
    Some(Plot1dSpec {
        expr: expr.clone(),
        var,
        start,
        end,
    })
}

fn extract_matlab_plot(expr: &Term, var: &Term, start: &Term, end: &Term) -> Option<Plot1dSpec> {
    let var = match var {
        Term::Atom(Atom::Symbol(s)) => s.clone(),
        _ => return None,
    };
    let start = number_from_term(start)?.to_f64_lossy()?;
    let end = number_from_term(end)?.to_f64_lossy()?;
    Some(Plot1dSpec {
        expr: expr.clone(),
        var,
        start,
        end,
    })
}
