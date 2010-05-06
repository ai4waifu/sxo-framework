//! Scaffold smoke: Form constructors + render (no oak).

use sxo_dialect_pari_gp::{GpForm, lower_request, parse_gp_form, render_gp};

#[test]
fn render_call_tree() {
    let form = GpForm::call(
        "factor",
        vec![GpForm::number_literal("6")],
    );
    assert_eq!(render_gp(&form), "factor(6)");
}

#[test]
fn parse_stub_errors() {
    let err = parse_gp_form("1+1").expect_err("parse must stay stubbed");
    assert!(err.message.contains("oak-pari"));
}

#[test]
fn lower_stub_errors() {
    let form = GpForm::number_literal("1");
    let err = lower_request(&form).expect_err("lower must stay stubbed");
    assert!(err.message.contains("lowering"));
}
