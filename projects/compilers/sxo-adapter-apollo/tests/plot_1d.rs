//! Integration tests for plot_1d.

use athena::{SampleDomain, SamplingPolicy, Session, push_app_named, push_int, push_symbol_name};
use sxo_adapter_apollo::plot_1d_svg;

#[test]
fn square_renders_polyline_svg() {
    let mut session = Session::new();
    let x = push_symbol_name(&mut session, "x");
    let two = push_int(&mut session, 2);
    let expr = push_app_named(&mut session, "Power", vec![x, two]);
    let svg = plot_1d_svg(&mut session, expr, "x", SampleDomain::new(-1.0, 1.0), SamplingPolicy::samples(32))
        .expect("svg");
    assert!(svg.contains("<svg"), "{svg}");
    assert!(svg.contains("<polyline") || svg.contains("<path"), "{svg}");
}
