//! Integration tests for plot_1d.

use athena::{
    ir::SemanticOperator,
    plot::SampleDomain,
    plot::SamplingPolicy,
    Session,
    runtime::values::arena::push_int,
    runtime::values::arena::push_semantic,
    runtime::values::arena::push_symbol_name,
};
use sxo_adapter_apollo::plot_1d_svg;

#[test]
fn square_renders_polyline_svg() {
    let mut session = Session::new();
    let x = push_symbol_name(&mut session, "x");
    let two = push_int(&mut session, 2);
    let expr = push_semantic(&mut session, SemanticOperator::Power, vec![x, two]);
    let svg = plot_1d_svg(&mut session, expr, "x", SampleDomain::new(-1.0, 1.0), SamplingPolicy::samples(32))
        .expect("svg");
    assert!(svg.contains("<svg"), "{svg}");
    assert!(svg.contains("<polyline") || svg.contains("<path"), "{svg}");
}
