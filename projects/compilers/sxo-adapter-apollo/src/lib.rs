//! Athena `SampledCurve` → Apollo `PlotSpec` / SVG 宿主适配。
//!
//! Apollo 不依赖 Athena；采样在 Athena，构图与渲染在 Apollo。

#![deny(missing_docs)]

use apollo::{ColumnTable, CompileOptions, LayerSpec, Mapping, PlotSpec, compile_plot, render_svg};
use athena::{
    Session,
    plot::{SampleDomain, SampledCurve, SamplingPolicy, sample_1d},
    types::{Diagnostic as AthenaDiagnostic, TermId},
};

/// 适配层错误（结构化 code 优先，文案仅调试）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// Athena 采样失败。
    Athena(AthenaDiagnostic),
    /// Apollo 数据 / 编译 / 渲染失败（稳定 code）。
    Apollo {
        /// `APOLLO_*` wire code。
        code: String,
    },
}

impl AdapterError {
    fn apollo(code: impl Into<String>) -> Self {
        Self::Apollo { code: code.into() }
    }
}

/// 将有效采样点转为列式表（gap 点丢弃；跨 gap 连线属后续多段层）。
pub fn sampled_curve_to_table(curve: &SampledCurve) -> Result<ColumnTable, AdapterError> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for p in &curve.points {
        if p.valid {
            xs.push(p.x);
            ys.push(p.y);
        }
    }
    if xs.len() < 2 {
        return Err(AdapterError::apollo("APOLLO_EMPTY_DATA"));
    }
    ColumnTable::new()
        .push_float("x", xs)
        .map_err(|d| AdapterError::apollo(d.code.as_str()))?
        .push_float("y", ys)
        .map_err(|d| AdapterError::apollo(d.code.as_str()))
}

/// 由采样曲线构造单层折线 `PlotSpec`。
pub fn line_plot_from_curve(curve: &SampledCurve) -> Result<PlotSpec, AdapterError> {
    let table = sampled_curve_to_table(curve)?;
    Ok(PlotSpec::new(table).mapping(Mapping::xy("x", "y")).layer(LayerSpec::geom_line()))
}

/// 编译并渲染为 SVG 字符串。
pub fn render_line_svg(curve: &SampledCurve) -> Result<String, AdapterError> {
    let plot = line_plot_from_curve(curve)?;
    let scene = compile_plot(&plot, CompileOptions::default()).map_err(|d| AdapterError::apollo(d.code.as_str()))?;
    render_svg(&scene).map_err(|d| AdapterError::apollo(d.code.as_str()))
}

/// 对一元表达式采样并渲染 SVG（1D plot 垂直切片）。
pub fn plot_1d_svg(
    session: &mut Session,
    expr: TermId,
    var: &str,
    domain: SampleDomain,
    policy: SamplingPolicy,
) -> Result<String, AdapterError> {
    let curve = sample_1d(session, expr, var, domain, policy).map_err(AdapterError::Athena)?;
    render_line_svg(&curve)
}
