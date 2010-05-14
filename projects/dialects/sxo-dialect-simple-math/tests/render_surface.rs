//! Simple Math Form render contract: lowercase / list / dict.

use sxo_dialect_simple_math::{DictKey, Expr, render};

#[test]
fn render_lowercase_call_list_dict() {
    let expr = Expr::add(
        Expr::pow(Expr::sin(Expr::var("x")), Expr::num(2.0)),
        Expr::pow(Expr::cos(Expr::var("x")), Expr::num(2.0)),
    );
    assert_eq!(render(&expr), "sin(x)^2 + cos(x)^2");

    let list = Expr::list(vec![Expr::num(1.0), Expr::num(2.0), Expr::var("x")]);
    assert_eq!(render(&list), "[1, 2, x]");

    let dict = Expr::dict(vec![
        (DictKey::Ident("a".into()), Expr::num(1.0)),
        (DictKey::String("b".into()), Expr::list(vec![Expr::num(2.0), Expr::num(3.0)])),
    ]);
    assert_eq!(render(&dict), "{a: 1, \"b\": [2, 3]}");
}
