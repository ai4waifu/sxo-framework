//! Integration tests for MATLAB parse (session arena `TermId`).

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session,
    runtime::values::arena::{push_bool, push_int, push_list, push_null, push_symbol_name},
    types::TermId,
};
use sxo_dialect_mathematica::{render, wexpr_from_session};
use sxo_dialect_matlab::{
    application_surface_name, lower_request, parse_matlab, push_matlab_call, render_matlab, try_plot_svg,
};

type Tid = TermId;

struct H {
    s: RefCell<Session>,
}

impl H {
    fn new() -> Self {
        Self { s: RefCell::new(Session::new()) }
    }

    fn parse(&self, input: &str) -> Tid {
        parse_matlab(&mut self.s.borrow_mut(), input).unwrap()
    }

    fn eval(&self, input: &str) -> Tid {
        let id = self.parse(input);
        let mut s = self.s.borrow_mut();
        let request = lower_request(&mut s, id);
        let engine = AthenaEngine::new();
        match engine.execute_request(&mut s, request) {
            Ok(result_id) => s.results.get(result_id).and_then(|r| r.symbolic_term).unwrap_or(id),
            Err(_) => id,
        }
    }

    fn eval_id(&self, id: Tid) -> Tid {
        let mut s = self.s.borrow_mut();
        let request = lower_request(&mut s, id);
        let engine = AthenaEngine::new();
        match engine.execute_request(&mut s, request) {
            Ok(result_id) => s.results.get(result_id).and_then(|r| r.symbolic_term).unwrap_or(id),
            Err(_) => id,
        }
    }

    fn i(&self, n: i64) -> Tid {
        push_int(&mut self.s.borrow_mut(), n)
    }

    fn sym(&self, name: &str) -> Tid {
        push_symbol_name(&mut self.s.borrow_mut(), name)
    }

    fn ap(&self, head: &str, args: Vec<Tid>) -> Tid {
        push_matlab_call(&mut self.s.borrow_mut(), head, args)
    }

    fn lst(&self, items: Vec<Tid>) -> Tid {
        push_list(&mut self.s.borrow_mut(), items)
    }

    fn boolean(&self, b: bool) -> Tid {
        push_bool(&mut self.s.borrow_mut(), b)
    }

    fn null(&self) -> Tid {
        push_null(&mut self.s.borrow_mut())
    }

    fn eq(&self, a: Tid, b: Tid) -> bool {
        self.s.borrow().arena.structural_eq(a, b)
    }

    fn render(&self, id: Tid) -> String {
        render_matlab(&self.s.borrow(), id)
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        f(&mut self.s.borrow_mut())
    }
}

#[test]
fn parse_plus_times() {
    let h = H::new();
    assert!(h.eq(h.eval("1 + 2 * 3"), h.i(7)));
}

#[test]
fn parse_array() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2 + 2]"), h.lst(vec![h.i(1), h.i(4)])));
}

#[test]
fn parse_call_sin() {
    let h = H::new();
    let t = h.parse("sin(x)");
    assert!(h.eq(t, h.ap("Sin", vec![h.sym("x")])));
}

#[test]
fn parse_power() {
    let h = H::new();
    let t = h.parse("x^3");
    assert!(h.eq(t, h.ap("Power", vec![h.sym("x"), h.i(3)])));
}

#[test]
fn parse_root_semicolon_returns_last() {
    let h = H::new();
    assert!(h.eq(h.eval("1; 2 + 2"), h.i(4)));
}

#[test]
fn parse_pythagorean() {
    let h = H::new();
    let t = h.parse("sin(x)^2 + cos(x)^2");
    let wrapped = h.ap("Simplify", vec![t]);
    assert!(h.eq(h.eval_id(wrapped), h.i(1)));
}

#[test]
fn parse_diff() {
    let h = H::new();
    let e = h.eval("diff(x^3, x)");
    let s = h.with_mut(|s| {
        let w = wexpr_from_session(s, e);
        render(&w)
    });
    assert!(s.contains('x'), "got {s}");
}

#[test]
fn parse_matrix_array() {
    let h = H::new();
    let t = h.parse("[1, 2; 3, 4]");
    assert!(h.eq(t, h.lst(vec![h.lst(vec![h.i(1), h.i(2)]), h.lst(vec![h.i(3), h.i(4)])])));
    assert_eq!(h.render(t), "[1, 2; 3, 4]");
}

#[test]
fn parse_integrate_and_sqrt() {
    let h = H::new();
    let e = h.eval("int(x^2, x)");
    let s = h.render(e);
    assert!(s.contains('x'), "got {s}");
    assert!(h.eq(h.eval("sqrt(9)"), h.i(3)));
}

#[test]
fn parse_comparison() {
    let h = H::new();
    assert!(h.eq(h.eval("3 > 2"), h.boolean(true)));
}

#[test]
fn parse_if_else_end() {
    let h = H::new();
    assert!(h.eq(h.eval("if 1, 2, else, 3, end"), h.i(2)));
}

#[test]
fn parse_while_false_skips_body() {
    let h = H::new();
    assert!(h.eq(h.eval("while 0, 1, end"), h.null()));
}

#[test]
fn parse_for_span_last() {
    let h = H::new();
    assert!(h.eq(h.eval("for i=1:3, i, end"), h.i(3)));
}

#[test]
fn parse_array_slice() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2, 3](1:2)"), h.lst(vec![h.i(1), h.i(2)])));
}

#[test]
fn parse_colon_step_flattens() {
    let h = H::new();
    assert!(h.eq(h.eval("1:2:10"), h.lst(vec![h.i(1), h.i(3), h.i(5), h.i(7), h.i(9)])));
}

#[test]
fn parse_mldivide_keeps_head() {
    let h = H::new();
    let t = h.parse(r"A\b");
    assert_eq!(application_surface_name(&h.s.borrow(), t).as_deref(), Some("LinearSolve"));
    // Symbolic operands stay residual under `LinearSolve`.
    let folded = h.eval_id(t);
    assert_eq!(application_surface_name(&h.s.borrow(), folded).as_deref(), Some("LinearSolve"));
    assert!(h.render(t).contains('\\'));
}

#[test]
fn parse_dot_times_distinct_head() {
    let h = H::new();
    let t = h.parse("x .* y");
    assert!(h.eq(t, h.ap("DotTimes", vec![h.sym("x"), h.sym("y")])));
    assert!(h.render(t).contains(".*"));
}

#[test]
fn parse_elementwise_ops_evaluate() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2].*[3, 4]"), h.lst(vec![h.i(3), h.i(8)])));
    assert!(h.eq(h.eval("2 .* [1, 2]"), h.lst(vec![h.i(2), h.i(4)])));
    assert!(h.eq(h.eval("[1, 2].^[2, 3]"), h.lst(vec![h.i(1), h.i(8)])));
    assert!(h.eq(h.eval("[1, 2, 3].^0"), h.lst(vec![h.i(1), h.i(1), h.i(1)])));
    assert!(h.eq(h.eval("[6, 8]./[2, 4]"), h.lst(vec![h.i(3), h.i(2)])));
    assert!(
        h.eq(h.eval("[1, 2; 3, 4].*[5, 6; 7, 8]"), h.lst(vec![h.lst(vec![h.i(5), h.i(12)]), h.lst(vec![h.i(21), h.i(32)]),]))
    );
    assert!(
        h.eq(h.eval("[1, 2; 3, 4]*[5, 6; 7, 8]"), h.lst(vec![h.lst(vec![h.i(19), h.i(22)]), h.lst(vec![h.i(43), h.i(50)]),]))
    );
}

#[test]
fn parse_matrix_linear_algebra() {
    let h = H::new();
    assert!(h.eq(h.eval("det([1, 2; 3, 4])"), h.i(-2)));
    assert!(h.eq(h.eval("sum([1, 2, 3])"), h.i(6)));
    assert!(h.eq(h.eval("sum([1, 2; 3, 4])"), h.lst(vec![h.i(4), h.i(6)])));
    // linsolve stays Extension until DomainGoal lowering (Living 27).
    let ls = h.parse("linsolve([1, 2; 3, 4], [5; 6])");
    assert_eq!(application_surface_name(&h.s.borrow(), ls).as_deref(), Some("LinearSolve"));
    assert_eq!(h.render(h.eval("det([1, 2; 3, 4])")), "-2");
}

#[test]
fn parse_end_index() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2, 3](end)"), h.i(3)));
}

#[test]
fn parse_assign_persists_in_sequence() {
    let h = H::new();
    assert!(h.eq(h.eval("x = 5; x + 1"), h.i(6)));
}

#[test]
fn parse_row_all_colon() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2; 3, 4](1,:)"), h.lst(vec![h.i(1), h.i(2)])));
}

#[test]
fn parse_col_all_colon() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2; 3, 4](:,2)"), h.lst(vec![h.i(2), h.i(4)])));
}

#[test]
fn parse_column_vector_and_mldivide_shape() {
    let h = H::new();
    let col = h.parse("[5; 6]");
    assert!(h.eq(col, h.lst(vec![h.lst(vec![h.i(5)]), h.lst(vec![h.i(6)])])));
    let t = h.parse("[1, 2; 3, 4] \\ [5; 6]");
    assert_eq!(application_surface_name(&h.s.borrow(), t).as_deref(), Some("LinearSolve"));
}

#[test]
fn parse_mldivide_2x2_stays_extension_until_goal() {
    let h = H::new();
    // `A\b` lowers to Extension LinearSolve — DomainGoal Solve is a later dialect wave.
    let e = h.parse("[1, 2; 3, 4] \\ [5; 6]");
    assert_eq!(application_surface_name(&h.s.borrow(), e).as_deref(), Some("LinearSolve"));
    assert!(h.render(e).contains('\\'));
}

#[test]
fn parse_matrix_constructors_and_size() {
    let h = H::new();
    let got = h.eval("eye(2)");
    let want = h.lst(vec![h.lst(vec![h.i(1), h.i(0)]), h.lst(vec![h.i(0), h.i(1)])]);
    assert!(h.eq(got, want), "got={} want={}", h.render(got), h.render(want));
    assert_eq!(h.render(h.eval("eye(2)")), "[1, 0; 0, 1]");

    assert!(
        h.eq(h.eval("zeros(2, 3)"), h.lst(vec![h.lst(vec![h.i(0), h.i(0), h.i(0)]), h.lst(vec![h.i(0), h.i(0), h.i(0)]),]))
    );
    assert_eq!(h.render(h.eval("zeros(2, 3)")), "[0, 0, 0; 0, 0, 0]");

    assert!(h.eq(h.eval("ones(2)"), h.lst(vec![h.lst(vec![h.i(1), h.i(1)]), h.lst(vec![h.i(1), h.i(1)]),])));
    assert_eq!(h.render(h.eval("ones(2)")), "[1, 1; 1, 1]");

    assert!(h.eq(h.eval("size([1, 2; 3, 4])"), h.lst(vec![h.i(2), h.i(2)])));
    assert_eq!(h.render(h.eval("size([1, 2; 3, 4])")), "[2, 2]");
    assert!(h.eq(h.eval("length([1, 2, 3])"), h.i(3)));
}

#[test]
fn parse_plot_negative_domain_renders_svg() {
    let h = H::new();
    let t = h.parse("plot(x^2, x, -1, 1)");
    let svg = h.with_mut(|s| try_plot_svg(s, t)).expect("extract").expect("render");
    assert!(svg.contains("<svg"), "got {svg}");
}
