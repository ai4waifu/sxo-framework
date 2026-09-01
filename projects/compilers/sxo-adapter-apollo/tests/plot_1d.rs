//! Integration tests for plot_1d.

use athena::{SampleDomain, SamplingPolicy, Term};
use sxo_adapter_apollo::plot_1d_svg;

#[test]
fn square_renders_polyline_svg() {
    let expr = Term::apply("Power", vec![Term::symbol("x"), Term::int(2)]);
    let svg = plot_1d_svg(&expr, "x", SampleDomain::new(-1.0, 1.0), SamplingPolicy::samples(32)).expect("svg");
    assert!(svg.contains("<svg"), "{svg}");
    assert!(svg.contains("<polyline") || svg.contains("<path"), "{svg}");
}
