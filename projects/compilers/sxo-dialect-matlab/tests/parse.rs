//! Integration tests for MATLAB parse.

use athena::{Term, evaluate};
use sxo_dialect_mathematica::{render, term_to_wexpr};
use sxo_dialect_matlab::{parse_matlab, render_matlab};

#[test]
fn parse_plus_times() {
    let t = parse_matlab("1 + 2 * 3").unwrap();
    assert_eq!(evaluate(&t), Term::int(7));
}

#[test]
fn parse_array() {
    let t = parse_matlab("[1, 2 + 2]").unwrap();
    assert_eq!(evaluate(&t), Term::List(vec![Term::int(1), Term::int(4)]));
}

#[test]
fn parse_call_sin() {
    let t = parse_matlab("sin(x)").unwrap();
    assert_eq!(t, Term::apply("Sin", vec![Term::symbol("x")]));
}

#[test]
fn parse_power() {
    let t = parse_matlab("x^3").unwrap();
    assert_eq!(t, Term::apply("Power", vec![Term::symbol("x"), Term::int(3)]));
}

#[test]
fn parse_root_semicolon_returns_last() {
    let t = parse_matlab("1; 2 + 2").unwrap();
    assert_eq!(evaluate(&t), Term::int(4));
}

#[test]
fn parse_pythagorean() {
    let t = parse_matlab("sin(x)^2 + cos(x)^2").unwrap();
    assert_eq!(evaluate(&Term::apply("Simplify", vec![t])), Term::int(1));
}

#[test]
fn parse_diff() {
    let t = parse_matlab("diff(x^3, x)").unwrap();
    let e = evaluate(&t);
    let s = render(&term_to_wexpr(&e));
    assert!(s.contains('x'), "got {s}");
}

#[test]
fn parse_matrix_array() {
    let t = parse_matlab("[1, 2; 3, 4]").unwrap();
    assert_eq!(
        t,
        Term::List(vec![Term::List(vec![Term::int(1), Term::int(2)]), Term::List(vec![Term::int(3), Term::int(4)]),])
    );
    assert_eq!(render_matlab(&t), "[1, 2; 3, 4]");
}

#[test]
fn parse_integrate_and_sqrt() {
    let t = parse_matlab("int(x^2, x)").unwrap();
    let e = evaluate(&t);
    let s = render_matlab(&e);
    assert!(s.contains('x'), "got {s}");

    let t = parse_matlab("sqrt(9)").unwrap();
    assert_eq!(evaluate(&t), Term::int(3));
}

#[test]
fn parse_comparison() {
    let t = parse_matlab("3 > 2").unwrap();
    assert_eq!(evaluate(&t), Term::boolean(true));
}

#[test]
fn parse_if_else_end() {
    let t = parse_matlab("if 1, 2, else, 3, end").unwrap();
    assert_eq!(evaluate(&t), Term::int(2));
}

#[test]
fn parse_while_false_skips_body() {
    let t = parse_matlab("while 0, 1, end").unwrap();
    assert_eq!(evaluate(&t), Term::null());
}

#[test]
fn parse_for_span_last() {
    let t = parse_matlab("for i=1:3, i, end").unwrap();
    assert_eq!(evaluate(&t), Term::int(3));
}

#[test]
fn parse_array_slice() {
    let t = parse_matlab("[1, 2, 3](1:2)").unwrap();
    assert_eq!(evaluate(&t), Term::List(vec![Term::int(1), Term::int(2)]));
}

#[test]
fn parse_colon_step_flattens() {
    let t = parse_matlab("1:2:10").unwrap();
    assert_eq!(
        evaluate(&t),
        Term::List(vec![Term::int(1), Term::int(3), Term::int(5), Term::int(7), Term::int(9)])
    );
}

#[test]
fn parse_mldivide_keeps_head() {
    use athena::{EvalKind, evaluate_outcome};

    let t = parse_matlab("A\\b").unwrap();
    assert_eq!(t.head_name(), Some("Mldivide"));
    let o = evaluate_outcome(&t);
    assert_eq!(o.kind, EvalKind::Unevaluated);
    assert!(o.diagnostics.is_empty(), "symbolic mldivide stays quiet unevaluated");
    assert!(render_matlab(&t).contains('\\'));
}

#[test]
fn parse_dot_times_distinct_head() {
    let t = parse_matlab("x .* y").unwrap();
    assert_eq!(t, Term::apply("DotTimes", vec![Term::symbol("x"), Term::symbol("y")]));
    assert!(render_matlab(&t).contains(".*"));
}

#[test]
fn parse_elementwise_ops_evaluate() {
    assert_eq!(
        evaluate(&parse_matlab("[1, 2].*[3, 4]").unwrap()),
        Term::List(vec![Term::int(3), Term::int(8)])
    );
    // Space before `.*` avoids `2.` float lexing.
    assert_eq!(
        evaluate(&parse_matlab("2 .* [1, 2]").unwrap()),
        Term::List(vec![Term::int(2), Term::int(4)])
    );
    assert_eq!(
        evaluate(&parse_matlab("[1, 2].^[2, 3]").unwrap()),
        Term::List(vec![Term::int(1), Term::int(8)])
    );
    assert_eq!(
        evaluate(&parse_matlab("[1, 2, 3].^0").unwrap()),
        Term::List(vec![Term::int(1), Term::int(1), Term::int(1)])
    );
    assert_eq!(
        evaluate(&parse_matlab("[6, 8]./[2, 4]").unwrap()),
        Term::List(vec![Term::int(3), Term::int(2)])
    );
    assert_eq!(
        evaluate(&parse_matlab("[1, 2; 3, 4].*[5, 6; 7, 8]").unwrap()),
        Term::List(vec![
            Term::List(vec![Term::int(5), Term::int(12)]),
            Term::List(vec![Term::int(21), Term::int(32)]),
        ])
    );
    // Matrix * stays Times (not silent elementwise)
    let t = parse_matlab("[1, 2; 3, 4]*[5, 6; 7, 8]").unwrap();
    assert_eq!(t.head_name(), Some("Times"));
    assert_eq!(evaluate(&t).head_name(), Some("Times"));
}

#[test]
fn parse_end_index() {
    let t = parse_matlab("[1, 2, 3](end)").unwrap();
    assert_eq!(evaluate(&t), Term::int(3));
}

#[test]
fn parse_assign_persists_in_sequence() {
    let t = parse_matlab("x = 5; x + 1").unwrap();
    assert_eq!(evaluate(&t), Term::int(6));
}

#[test]
fn parse_row_all_colon() {
    // `A(1,:)` on a matrix → first row
    let t = parse_matlab("[1, 2; 3, 4](1,:)").unwrap();
    assert_eq!(evaluate(&t), Term::List(vec![Term::int(1), Term::int(2)]));
}

#[test]
fn parse_col_all_colon() {
    // `A(:,2)` → second column as a list of row picks
    let t = parse_matlab("[1, 2; 3, 4](:,2)").unwrap();
    assert_eq!(evaluate(&t), Term::List(vec![Term::int(2), Term::int(4)]));
}

#[test]
fn parse_column_vector_and_mldivide_shape() {
    let col = parse_matlab("[5; 6]").unwrap();
    assert_eq!(
        col,
        Term::List(vec![Term::List(vec![Term::int(5)]), Term::List(vec![Term::int(6)])])
    );
    let t = parse_matlab("[1, 2; 3, 4] \\ [5; 6]").unwrap();
    assert_eq!(t.head_name(), Some("Mldivide"));
}

#[test]
fn parse_mldivide_2x2_evaluates() {
    let t = parse_matlab("[1, 2; 3, 4] \\ [5; 6]").unwrap();
    let e = evaluate(&t);
    assert_eq!(
        e,
        Term::List(vec![
            Term::List(vec![Term::int(-4)]),
            Term::List(vec![Term::rational_i64(9, 2).unwrap()]),
        ])
    );
}

#[test]
fn parse_matrix_constructors_and_size() {
    assert_eq!(
        evaluate(&parse_matlab("eye(2)").unwrap()),
        Term::List(vec![
            Term::List(vec![Term::int(1), Term::int(0)]),
            Term::List(vec![Term::int(0), Term::int(1)]),
        ])
    );
    assert_eq!(render_matlab(&evaluate(&parse_matlab("eye(2)").unwrap())), "[1, 0; 0, 1]");

    assert_eq!(
        evaluate(&parse_matlab("zeros(2, 3)").unwrap()),
        Term::List(vec![
            Term::List(vec![Term::int(0), Term::int(0), Term::int(0)]),
            Term::List(vec![Term::int(0), Term::int(0), Term::int(0)]),
        ])
    );
    assert_eq!(render_matlab(&evaluate(&parse_matlab("zeros(2, 3)").unwrap())), "[0, 0, 0; 0, 0, 0]");

    assert_eq!(
        evaluate(&parse_matlab("ones(2)").unwrap()),
        Term::List(vec![
            Term::List(vec![Term::int(1), Term::int(1)]),
            Term::List(vec![Term::int(1), Term::int(1)]),
        ])
    );
    assert_eq!(render_matlab(&evaluate(&parse_matlab("ones(2)").unwrap())), "[1, 1; 1, 1]");

    assert_eq!(
        evaluate(&parse_matlab("size([1, 2; 3, 4])").unwrap()),
        Term::List(vec![Term::int(2), Term::int(2)])
    );
    assert_eq!(render_matlab(&evaluate(&parse_matlab("size([1, 2; 3, 4])").unwrap())), "[2, 2]");

    assert_eq!(evaluate(&parse_matlab("length([1, 2, 3])").unwrap()), Term::int(3));
}
