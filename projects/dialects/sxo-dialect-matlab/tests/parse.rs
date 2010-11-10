//! Integration tests for MATLAB parse (session arena `TermId`).

use std::cell::RefCell;

use athena::{
    AthenaEngine, Session,
    runtime::values::arena::{push_bool, push_int, push_list, push_null, push_symbol_name},
    types::TermId,
};
use sxo_dialect_matlab::{
    MatlabAtom, MatlabForm, application_surface_name, form_to_term, install_session_conventions, lower_request,
    parse_matlab, parse_matlab_form, push_matlab_call, render_matlab, render_matlab_form, try_plot_svg,
};

type Tid = TermId;

struct H {
    s: RefCell<Session>,
}

impl H {
    fn new() -> Self {
        let mut s = Session::new();
        install_session_conventions(&mut s);
        Self { s: RefCell::new(s) }
    }

    fn parse(&self, input: &str) -> Tid {
        parse_matlab(&mut self.s.borrow_mut(), input).unwrap()
    }

    fn eval(&self, input: &str) -> Tid {
        let form = parse_matlab_form(input).unwrap();
        let mut s = self.s.borrow_mut();
        let request = lower_request(&mut s, &form);
        let engine = AthenaEngine::new();
        match engine.execute_request(&mut s, request) {
            Ok(result_id) => {
                let result = s.results.get(result_id).unwrap_or_else(|| panic!("missing ResultId after eval: {input}"));
                result.symbolic_term.unwrap_or_else(|| {
                    panic!(
                        "result has no symbolic_term after eval: {input} (status={}, coverage={})",
                        result.status.name(),
                        result.coverage.name()
                    )
                })
            }
            Err(err) => panic!("execute_request failed for {input}: {err}"),
        }
    }

    fn eval_form(&self, form: &MatlabForm) -> Tid {
        let mut s = self.s.borrow_mut();
        let request = lower_request(&mut s, form);
        let engine = AthenaEngine::new();
        match engine.execute_request(&mut s, request) {
            Ok(result_id) => {
                let result = s.results.get(result_id).unwrap_or_else(|| panic!("missing ResultId after eval_form"));
                result.symbolic_term.unwrap_or_else(|| {
                    panic!(
                        "result has no symbolic_term after eval_form (status={}, coverage={})",
                        result.status.name(),
                        result.coverage.name()
                    )
                })
            }
            Err(err) => panic!("execute_request failed for eval_form: {err}"),
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
    let body = parse_matlab_form("sin(x)^2 + cos(x)^2").unwrap();
    let wrapped = MatlabForm::call("Simplify", vec![body]);
    assert!(h.eq(h.eval_form(&wrapped), h.i(1)));
}

#[test]
fn parse_diff() {
    let h = H::new();
    let e = h.eval("diff(x^3, x)");
    let s = h.render(e);
    assert!(s.contains('x'), "got {s}");
    assert_eq!(h.render(h.eval("diff(x^2, x, 2)")), "2");
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
fn parse_switch_case_otherwise() {
    let h = H::new();
    assert_eq!(h.render(h.eval("switch 1, case 1, 2, otherwise, 3, end")), "2");
    assert_eq!(h.render(h.eval("switch 2, case 1, 2, otherwise, 3, end")), "3");
    assert_eq!(h.render(h.eval("switch 1, case 2, 9, case 1, 4, otherwise, 0, end")), "4");
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
    let form = parse_matlab_form(r"A\b").unwrap();
    let t = form_to_term(&mut h.s.borrow_mut(), &form);
    assert_eq!(application_surface_name(&h.s.borrow(), t).as_deref(), Some("LinearSolve"));
    // Unbound symbolic operands stay residual under `LinearSolve` Own projection.
    let folded = h.eval_form(&form);
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
fn parse_transpose_forms_distinct() {
    let nonconj = parse_matlab_form("[1, 2].'").unwrap();
    assert_eq!(nonconj, MatlabForm::call("Transpose", vec![MatlabForm::list(vec![MatlabForm::int(1), MatlabForm::int(2)])]));
    let conj = parse_matlab_form("x'").unwrap();
    assert_eq!(conj, MatlabForm::call("ConjugateTranspose", vec![MatlabForm::symbol("x")]));
}

#[test]
fn parse_transpose_evaluates_nested_lists() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2].'"), h.lst(vec![h.lst(vec![h.i(1)]), h.lst(vec![h.i(2)])])));
    assert!(h.eq(h.eval("[1; 2].'"), h.lst(vec![h.i(1), h.i(2)])));
    assert!(h.eq(h.eval("[1, 2; 3, 4].'"), h.lst(vec![h.lst(vec![h.i(1), h.i(3)]), h.lst(vec![h.i(2), h.i(4)])])));
    // Real conjugate transpose matches transpose on literals.
    assert!(h.eq(h.eval("[1, 2; 3, 4]'"), h.lst(vec![h.lst(vec![h.i(1), h.i(3)]), h.lst(vec![h.i(2), h.i(4)])])));
}

#[test]
fn set_2d_matrix_binds_domain_object() {
    let h = H::new();
    let form = parse_matlab_form("A = [1, 2; 3, 4]").unwrap();
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &form);
    assert!(matches!(request, athena::api::AthenaRequest::Command(athena::api::SessionCommand::DefineMatrix { .. })));
    AthenaEngine::new().execute_request(&mut s, request).expect("define matrix");
    let symbol = s.arena.symbols_mut().intern("A");
    assert!(s.matrix_binding(symbol).is_some());
    assert!(s.defs.binding(symbol).is_none());
}

#[test]
fn transpose_symbol_after_2d_set_uses_matrix_binding() {
    let h = H::new();
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; A.'")), "[1, 3; 2, 4]");
}

#[test]
fn read_matrix_binding_renders_nested_list() {
    let h = H::new();
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; A")), "[1, 2; 3, 4]");
}

#[test]
fn det_symbol_after_2d_set() {
    let h = H::new();
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; det(A)")), "-2");
}

#[test]
fn inv_literal_and_singular_residual() {
    let h = H::new();
    assert_eq!(h.render(h.eval("inv([1, 0; 0, 2])")), "[1, 0; 0, 1/2]");
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; inv(A)")), "[-2, 1; 3/2, -1/2]");
    // Living 16: singular Inverse → Inverse[Singular] residual (rendered as inv(...)).
    let singular = h.render(h.eval("inv([1, 2; 2, 4])"));
    assert!(
        singular.contains("inv") || singular.contains("Inverse") || singular.contains("Singular"),
        "expected Inverse Singular residual, got {singular}"
    );
}

#[test]
fn rank_trace_rref_literal_goals() {
    let h = H::new();
    assert_eq!(h.render(h.eval("rank([1, 2; 2, 4])")), "1");
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; rank(A)")), "2");
    assert_eq!(h.render(h.eval("trace([1, 2; 3, 4])")), "5");
    assert_eq!(h.render(h.eval("rref([1, 2; 2, 4])")), "[1, 2; 0, 0]");
}

#[test]
fn norm_dot_cross_literal_goals() {
    let h = H::new();
    assert_eq!(h.render(h.eval("norm([3, 4])")), "5");
    assert_eq!(h.render(h.eval("dot([1, 2], [3, 4])")), "11");
    assert_eq!(h.render(h.eval("cross([1, 0, 0], [0, 1, 0])")), "[0, 0, 1]");
}

#[test]
fn null_literal_column_basis_goals() {
    let h = H::new();
    // Living 16: MATLAB `null` → column basis `[[-2];[1]]`.
    assert_eq!(h.render(h.eval("null([1, 2; 2, 4])")), "[-2; 1]");
    assert_eq!(h.render(h.eval("A = [1, 2; 2, 4]; null(A)")), "[-2; 1]");
    // Full-rank via `eye` Form constructor → empty.
    assert_eq!(h.render(h.eval("null(eye(2))")), "[]");
}

#[test]
fn diag_and_cond_literal_goals() {
    let h = H::new();
    assert_eq!(h.render(h.eval("diag([1, 2])")), "[1, 0; 0, 2]");
    // Living 16: `cond` → ConditionNumber Goal (LU pivot-ratio estimate).
    let c = h.render(h.eval("cond([2, 0; 0, 2])"));
    assert!(
        c == "1" || c.starts_with("1.") || c == "1.0",
        "expected ~1 conditioning, got {c}"
    );
    let singular = h.render(h.eval("cond([1, 2; 2, 4])"));
    assert!(
        singular.contains("Inf") || singular.contains("inf") || singular.contains("Infinity"),
        "expected Inf for singular, got {singular}"
    );
}

#[test]
fn linsolve_symbol_after_2d_set() {
    let h = H::new();
    // Column `b` is matrix Own; Solve resolves both bindings.
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; b = [5; 6]; A\\b")), "[-4; 9/2]");
    // Living 16: inconsistent → empty; Infinite → particular column (free_vars in evidence).
    assert_eq!(h.render(h.eval("[1, 2; 2, 4] \\ [1; 0]")), "[]");
    assert_eq!(h.render(h.eval("[1, 2; 2, 4] \\ [2; 4]")), "[2; 0]");
    // Living 16: machine-float Form → MachineSolve Singular residual.
    let singular = h.render(h.eval("[1.0, 2.0; 2.0, 4.0] \\ [1.0; 0.0]"));
    assert!(
        singular.contains("linsolve") || singular.contains("\\") || singular.contains("Singular"),
        "expected Singular residual, got {singular}"
    );
}

#[test]
fn times_symbols_after_2d_set() {
    let h = H::new();
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; B = [5, 6; 7, 8]; A*B")), "[19, 22; 43, 50]");
    assert_eq!(h.render(h.eval("A = [1, 2; 3, 4]; B = [5, 6; 7, 8]; A.*B")), "[5, 12; 21, 32]");
    assert_eq!(h.render(h.eval("A = [6, 8; 10, 12]; B = [2, 4; 5, 6]; A./B")), "[3, 2; 2, 2]");
    assert_eq!(h.render(h.eval("A = [2, 3; 4, 5]; B = [2, 2; 2, 2]; A.^B")), "[4, 9; 16, 25]");
    // Living 16: matrix `/` is RightSolve (mrdivide), not element-wise.
    assert_eq!(h.render(h.eval("[1, 2] / [1, 2; 3, 4]")), "[[1, 0]]");
    assert_eq!(h.render(h.eval("[1, 2; 3, 4] / [1, 2; 3, 4]")), "[1, 0; 0, 1]");
}

#[test]
fn parse_mrdivide_2x2_lowers_to_right_solve_goal() {
    use athena::api::{AthenaRequest, DomainGoal};

    let form = parse_matlab_form("[1, 2] / [1, 2; 3, 4]").unwrap();
    let mut session = athena::runtime::Session::new();
    let request = lower_request(&mut session, &form);
    assert!(matches!(
        request,
        AthenaRequest::Goal(DomainGoal::Dispatch(
            athena::domains::DomainRequest::LinearAlgebra(athena::domains::linear_algebra::LinearAlgebraRequest::RightSolve { .. })
        ))
    ));
}

#[test]
fn set_row_vector_binds_matrix_own() {
    let h = H::new();
    let form = parse_matlab_form("A = [10, 20]").unwrap();
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &form);
    assert!(matches!(request, athena::api::AthenaRequest::Command(athena::api::SessionCommand::DefineMatrix { .. })));
    AthenaEngine::new().execute_request(&mut s, request).expect("define row");
    let symbol = s.arena.symbols_mut().intern("A");
    assert!(s.matrix_binding(symbol).is_some());
    drop(s);
    assert!(h.eq(h.eval("A(2)"), h.i(20)));
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
        h.eq(h.eval("[6, 8; 10, 12]./[2, 4; 5, 6]"), h.lst(vec![h.lst(vec![h.i(3), h.i(2)]), h.lst(vec![h.i(2), h.i(2)]),]))
    );
    assert!(
        h.eq(h.eval("[2, 3; 4, 5].^[2, 2; 2, 2]"), h.lst(vec![h.lst(vec![h.i(4), h.i(9)]), h.lst(vec![h.i(16), h.i(25)]),]))
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
    assert!(h.eq(h.eval("prod([2, 3, 4])"), h.i(24)));
    assert!(h.eq(h.eval("prod([1, 2; 3, 4])"), h.lst(vec![h.i(3), h.i(8)])));
    // Living 16: `linsolve` / `\` Form literals lower to Solve Goal (not residual Extension).
    assert_eq!(h.render(h.eval("linsolve([1, 2; 3, 4], [5; 6])")), "[-4; 9/2]");
    assert_eq!(h.render(h.eval("det([1, 2; 3, 4])")), "-2");
}

#[test]
fn parse_end_index() {
    let h = H::new();
    assert!(h.eq(h.eval("[1, 2, 3](end)"), h.i(3)));
}

#[test]
fn unary_minus_binds_looser_than_power() {
    // MATLAB: `-x^2` → `-(x^2)`, not `(-x)^2` (which would simplify to `x^2`).
    let form = parse_matlab_form("-x^2").unwrap();
    assert_eq!(
        form,
        MatlabForm::call(
            "Minus",
            vec![MatlabForm::call("Power", vec![MatlabForm::symbol("x"), MatlabForm::int(2)])]
        )
    );
    assert_eq!(render_matlab_form(&form), "-x^2");
    let h = H::new();
    assert_eq!(h.render(h.eval("exp(-x^2)")), "exp(-x^2)");
    assert_eq!(h.render(h.eval("fourier(exp(-x^2))")), "fourier(exp(-x^2))");
}

#[test]
fn double_unary_plus_is_identity_not_increment() {
    // MATLAB has no `++` operator. `++A` is unary `+` twice and equals `A`.
    let form = parse_matlab_form("++A").unwrap();
    assert_eq!(form, MatlabForm::symbol("A"));
    let h = H::new();
    assert_eq!(h.render(h.eval("++A")), "A");
    // Postfix `A++` is not MATLAB (oak error), covered by matrix notes.
}

#[test]
fn parse_matrix_linear_index_column_major() {
    let h = H::new();
    // [1,2; 3,4] column-major linear: 1,3,2,4
    assert!(h.eq(h.eval("[1, 2; 3, 4](1)"), h.i(1)));
    assert!(h.eq(h.eval("[1, 2; 3, 4](2)"), h.i(3)));
    assert!(h.eq(h.eval("[1, 2; 3, 4](3)"), h.i(2)));
    assert!(h.eq(h.eval("[1, 2; 3, 4](4)"), h.i(4)));
    assert!(h.eq(h.eval("[1, 2; 3, 4](1, 2)"), h.i(2)));
}

#[test]
fn parse_assign_then_index_own_binding() {
    let h = H::new();
    let form = parse_matlab_form("A(2)").unwrap();
    assert_eq!(form, MatlabForm::call("Part", vec![MatlabForm::symbol("A"), MatlabForm::int(2)]));
    assert!(h.eq(h.eval("A = [10, 20]; A(2)"), h.i(20)));
    assert!(h.eq(h.eval("M = [1, 2; 3, 4]; M(2)"), h.i(3)));
}

#[test]
fn indexed_assignment_updates_own_binding() {
    let h = H::new();
    let form = parse_matlab_form("A(2) = 9").unwrap();
    assert_eq!(
        form,
        MatlabForm::call(
            "Set",
            vec![
                MatlabForm::call("Part", vec![MatlabForm::symbol("A"), MatlabForm::int(2)]),
                MatlabForm::int(9),
            ]
        )
    );
    assert_eq!(h.render(h.eval("A = [1, 2, 3]; A(2) = 9; A")), "[1, 9, 3]");
}

#[test]
fn indexed_assignment_updates_matrix_cell() {
    let h = H::new();
    assert_eq!(h.render(h.eval("M = [1, 2; 3, 4]; M(1, 2) = 9; M")), "[1, 9; 3, 4]");
}

#[test]
fn indexed_assignment_grows_with_end_plus_and_pad() {
    let h = H::new();
    assert_eq!(h.render(h.eval("B = 1:4; B(end+1) = 5; B")), "[1, 2, 3, 4, 5]");
    assert_eq!(h.render(h.eval("A = [1, 2, 3]; A(5) = 9; A")), "[1, 2, 3, 0, 9]");
    assert_eq!(
        h.render(h.eval("M = zeros(2); M(3, 3) = 1; M")),
        "[0, 0, 0; 0, 0, 0; 0, 0, 1]"
    );
}

#[test]
fn set_integer_range_binds_matrix_own() {
    let h = H::new();
    let form = parse_matlab_form("B = 1:4").unwrap();
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &form);
    assert!(matches!(request, athena::api::AthenaRequest::Command(athena::api::SessionCommand::DefineMatrix { .. })));
    AthenaEngine::new().execute_request(&mut s, request).expect("define range");
    let symbol = s.arena.symbols_mut().intern("B");
    assert!(s.matrix_binding(symbol).is_some());
    drop(s);
    assert_eq!(h.render(h.eval("B(end+1) = 5; B")), "[1, 2, 3, 4, 5]");
}

#[test]
fn parse_call_vs_part_disambiguation() {
    // Known math heads stay calls even with index-shaped args.
    assert_eq!(parse_matlab_form("sin(0)").unwrap().head_name(), Some("Sin"));
    // Unknown head + numeric args → Part (subsref), not a free call.
    assert_eq!(parse_matlab_form("A(2)").unwrap(), MatlabForm::call("Part", vec![MatlabForm::symbol("A"), MatlabForm::int(2)]));
    // Symbol args do not look like subsref → stay as call head.
    assert_eq!(parse_matlab_form("f(x)").unwrap().head_name(), Some("f"));
}

#[test]
fn free_symbol_index_shaped_call_keeps_args_on_eval() {
    let h = H::new();
    // Parse still chooses Part for unknown+numeric (MATLAB `()` shared with subsref).
    assert_eq!(
        parse_matlab_form("speye(2)").unwrap(),
        MatlabForm::call("Part", vec![MatlabForm::symbol("speye"), MatlabForm::int(2)])
    );
    // Athena Index residual on free symbol must not strip to bare `speye`.
    assert_eq!(h.render(h.eval("speye(2)")), "speye(2)");
}

#[test]
fn spfun_keeps_function_handle_and_speye_args() {
    let form = parse_matlab_form("spfun(@sqrt, speye(2))").unwrap();
    match form {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "spfun");
            assert_eq!(args.len(), 2, "got {args:?}");
            assert_eq!(args[0].head_name(), Some("FunctionHandle"));
            assert_eq!(
                args[1],
                MatlabForm::call("Part", vec![MatlabForm::symbol("speye"), MatlabForm::int(2)])
            );
        }
        other => panic!("expected spfun call, got {other:?}"),
    }
    let h = H::new();
    assert_eq!(h.render(h.eval("spfun(@sqrt, speye(2))")), "spfun(@Sqrt, speye(2))");
}

#[test]
fn function_handle_args_kept_in_bsxfun_and_arrayfun() {
    let bsx = parse_matlab_form("bsxfun(@plus, [1, 2], [3; 4])").unwrap();
    match bsx {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "bsxfun");
            assert_eq!(args.len(), 3, "got {args:?}");
            assert_eq!(args[0].head_name(), Some("FunctionHandle"));
        }
        other => panic!("expected bsxfun call, got {other:?}"),
    }
    let h = H::new();
    assert_eq!(
        h.render(h.eval("bsxfun(@plus, [1, 2], [3; 4])")),
        "bsxfun(@plus, [1, 2], [3; 4])"
    );

    let af = parse_matlab_form("arrayfun(@sin, [0])").unwrap();
    assert_eq!(af.head_name(), Some("arrayfun"));
    match af {
        MatlabForm::Call { args, .. } => {
            assert_eq!(args[0].head_name(), Some("FunctionHandle"));
        }
        other => panic!("expected arrayfun call, got {other:?}"),
    }
    assert_eq!(h.render(h.eval("arrayfun(@sin, [0])")), "arrayfun(@Sin, [0])");
}

#[test]
fn parse_anonymous_function_handle_and_call() {
    let form = parse_matlab_form("@(x) x^2").unwrap();
    assert_eq!(form.head_name(), Some("Function"));
    let handle = parse_matlab_form("@sin").unwrap();
    assert_eq!(handle, MatlabForm::call("FunctionHandle", vec![MatlabForm::symbol("Sin")]));
    let h = H::new();
    assert_eq!(h.render(h.eval("f = @(x) x^2; f(4)")), "16");
    assert_eq!(h.render(h.eval("feval(@sin, 0)")), "0");
}

#[test]
fn parse_matrix_colon_all_column_major_flatten() {
    let h = H::new();
    let form = parse_matlab_form("[1, 2; 3, 4](:)").unwrap();
    assert_eq!(
        form,
        MatlabForm::call(
            "Part",
            vec![
                MatlabForm::list(vec![
                    MatlabForm::list(vec![MatlabForm::int(1), MatlabForm::int(2)]),
                    MatlabForm::list(vec![MatlabForm::int(3), MatlabForm::int(4)]),
                ]),
                MatlabForm::symbol(":"),
            ]
        )
    );
    // Column-major flatten → 4×1 nested column vector.
    let expected = h.lst(vec![h.lst(vec![h.i(1)]), h.lst(vec![h.i(3)]), h.lst(vec![h.i(2)]), h.lst(vec![h.i(4)])]);
    assert!(h.eq(h.eval("[1, 2; 3, 4](:)"), expected.clone()));
    assert!(h.eq(h.eval("A = [1, 2; 3, 4]; A(:)"), expected));
    assert_eq!(h.render(h.eval("[1, 2; 3, 4](:)")), "[1; 3; 2; 4]");
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
fn parse_mldivide_2x2_lowers_to_solve_goal() {
    let h = H::new();
    // Living 16: literal `\` / `linsolve` lower to `LinearAlgebraRequest::Solve` (ExactSolve `MatrixResult`).
    let form = parse_matlab_form("[1, 2; 3, 4] \\ [5; 6]").unwrap();
    let mut s = h.s.borrow_mut();
    let request = lower_request(&mut s, &form);
    assert!(matches!(
        request,
        athena::api::AthenaRequest::Goal(athena::api::DomainGoal::Dispatch(
            athena::domains::DomainRequest::LinearAlgebra(athena::domains::linear_algebra::LinearAlgebraRequest::Solve { .. })
        ))
    ));
    drop(s);
    assert_eq!(h.render(h.eval("[1, 2; 3, 4] \\ [5; 6]")), "[-4; 9/2]");
    assert_eq!(h.render(h.eval("linsolve([1, 2; 3, 4], [5; 6])")), "[-4; 9/2]");
    assert_eq!(h.render(h.eval("A = eye(2); b = [3; 5]; A\\b")), "[3; 5]");
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

    // Living 16: rectangular eye interns typed MatrixRef (leading diagonal ones).
    assert_eq!(h.render(h.eval("eye(2, 3)")), "[1, 0, 0; 0, 1, 0]");
    assert_eq!(h.render(h.eval("A = eye(2); size(A)")), "[2, 2]");

    assert!(h.eq(h.eval("size([1, 2; 3, 4])"), h.lst(vec![h.i(2), h.i(2)])));
    assert_eq!(h.render(h.eval("size([1, 2; 3, 4])")), "[2, 2]");
    assert!(h.eq(h.eval("length([1, 2, 3])"), h.i(3)));
}

#[test]
fn size_of_matrix_binding() {
    let h = H::new();
    // Living 16: bound MatrixRef shape without nested-list reverse recognition.
    assert_eq!(h.render(h.eval("A = [1, 2, 3; 4, 5, 6]; size(A)")), "[2, 3]");
}

#[test]
fn length_of_row_vector_binding() {
    let h = H::new();
    // 1×n Own projects as flat list; Length follows columns.
    assert_eq!(h.render(h.eval("A = [10, 20, 30]; length(A)")), "3");
}

#[test]
fn cumsum_on_row_vector() {
    let h = H::new();
    // Living 16: cumsum → Accumulate on typed 1×n MatrixRef.
    assert_eq!(h.render(h.eval("cumsum([1, 2, 3])")), "[1, 3, 6]");
    assert_eq!(h.render(h.eval("A = [1, 2, 3]; cumsum(A)")), "[1, 3, 6]");
}

#[test]
fn prod_on_matrix_binding() {
    let h = H::new();
    // Living 16: prod → Product on typed MatrixRef (column products / row scalar).
    assert_eq!(h.render(h.eval("A = [2, 3, 4]; prod(A)")), "24");
    assert_eq!(h.render(h.eval("B = [1, 2; 3, 4]; prod(B)")), "[3, 8]");
}

#[test]
fn parse_plot_negative_domain_renders_svg() {
    let h = H::new();
    let t = h.parse("plot(x^2, x, -1, 1)");
    let svg = h.with_mut(|s| try_plot_svg(s, t)).expect("extract").expect("render");
    assert!(svg.contains("<svg"), "got {svg}");
}

#[test]
fn render_keeps_parens_for_negative_rational_power() {
    let h = H::new();
    let roundtrip = h.render(h.eval("(-8)^(1/3)"));
    assert_eq!(roundtrip, "-2", "got {roundtrip}");
}

#[test]
fn render_keeps_parens_for_sum_power_base() {
    let h = H::new();
    let id = h.parse("(x+1)^2");
    assert_eq!(h.render(id), "(x + 1)^2", "got {}", h.render(id));
    // Eval may commute Plus arguments; must not expand away the Power wrapper.
    let got = h.render(h.eval("(x+1)^2"));
    assert_eq!(got, "(1 + x)^2", "got {got}");
}

#[test]
fn elementwise_less_vector_scalar() {
    let h = H::new();
    assert_eq!(h.render(h.eval("[1, 2, 3] < 2")), "[true, false, false]");
    assert_eq!(h.render(h.eval("[1, 2, 3] > 2")), "[false, false, true]");
    assert_eq!(h.render(h.eval("[1, 2, 3] >= 2")), "[false, true, true]");
}

#[test]
fn elementwise_unequal_vector_mask() {
    let h = H::new();
    assert_eq!(h.render(h.eval("[1, 2] ~= [1, 3]")), "[false, true]");
    assert_eq!(h.render(h.eval("[1, 2] == [1, 3]")), "[true, false]");
}

#[test]
fn bool_atoms_short_circuit_and_or() {
    let h = H::new();
    assert_eq!(h.render(h.eval("true && false")), "false", "got {}", h.render(h.eval("true && false")));
    assert_eq!(h.render(h.eval("true || false")), "true");
    assert_eq!(h.render(h.eval("false && 1")), "false");
}

#[test]
fn and_or_short_circuit_skips_rhs_assignment() {
    let h = H::new();
    // false && (x=1) must not define x.
    assert_eq!(h.render(h.eval("false && (x = 1)")), "false");
    assert_eq!(h.render(h.eval("x")), "x");

    // true || (y=1) must not define y.
    assert_eq!(h.render(h.eval("true || (y = 1)")), "true");
    assert_eq!(h.render(h.eval("y")), "y");

    // When the leading arm allows, assignment still runs.
    assert_eq!(h.render(h.eval("true && (z = 7)")), "7");
    assert_eq!(h.render(h.eval("z")), "7");
}

#[test]
fn scalar_or_and_short_circuit_ops() {
    let h = H::new();
    assert_eq!(parse_matlab_form("1 | 0").unwrap().head_name(), Some("ElementwiseOr"));
    assert_eq!(parse_matlab_form("1 & 0").unwrap().head_name(), Some("ElementwiseAnd"));
    assert_eq!(parse_matlab_form("1 || 0").unwrap().head_name(), Some("Or"));
    assert_eq!(parse_matlab_form("1 && 0").unwrap().head_name(), Some("And"));
    assert_eq!(h.render(h.eval("1 | 0")), "true", "got {}", h.render(h.eval("1 | 0")));
    assert_eq!(h.render(h.eval("1 & 0")), "false");
    assert_eq!(h.render(h.eval("1 || 0")), "true");
    assert_eq!(h.render(h.eval("1 && 0")), "false");
    assert_eq!(h.render(h.eval("[1, 0] | [0, 1]")), "[true, true]", "got {}", h.render(h.eval("[1, 0] | [0, 1]")));
    assert_eq!(h.render(h.eval("[1, 0] & [1, 1]")), "[true, false]", "got {}", h.render(h.eval("[1, 0] & [1, 1]")));
}

#[test]
fn parse_matlab_form_without_session() {
    let form = parse_matlab_form("1 + 2 * 3").unwrap();
    assert_eq!(form.head_name(), Some("Plus"));
    match form {
        MatlabForm::Call { args, .. } => {
            assert!(matches!(args[0], MatlabForm::Atom(MatlabAtom::Number(_))));
            assert_eq!(args[1].head_name(), Some("Times"));
        }
        other => panic!("expected Plus call, got {other:?}"),
    }
}

#[test]
fn render_matlab_form_without_arena() {
    let sum = parse_matlab_form("1 + 2 * 3").unwrap();
    assert_eq!(render_matlab_form(&sum), "1 + 2*3");
    let part = parse_matlab_form("A(2)").unwrap();
    assert_eq!(render_matlab_form(&part), "A(2)");
    let anon = parse_matlab_form("@(x) x^2").unwrap();
    assert_eq!(render_matlab_form(&anon), "@(x) x^2");
    let handle = parse_matlab_form("@sin").unwrap();
    assert_eq!(render_matlab_form(&handle), "@Sin");
}

#[test]
fn command_syntax_lowers_to_command_form() {
    for (input, name, arg) in [
        ("hold on", "hold", "on"),
        ("hold on;", "hold", "on"),
        ("grid on", "grid", "on"),
        ("syms x", "syms", "x"),
        ("axis equal", "axis", "equal"),
        ("close all", "close", "all"),
        ("colormap jet", "colormap", "jet"),
        ("which sin", "which", "sin"),
        ("profile on", "profile", "on"),
        ("format long", "format", "long"),
    ] {
        let form = parse_matlab_form(input).unwrap_or_else(|e| panic!("{input:?}: {e}"));
        match form {
            MatlabForm::Call { head, args } => {
                assert_eq!(head, "Command", "{input:?}");
                assert_eq!(args.len(), 2, "{input:?} => {args:?}");
                assert!(args[0].is_symbol(name), "{input:?} => {:?}", args[0]);
                assert!(args[1].is_symbol(arg), "{input:?} => {:?}", args[1]);
            }
            other => panic!("{input:?}: expected Command, got {other:?}"),
        }
        assert_eq!(render_matlab_form(&parse_matlab_form(input).unwrap()), format!("{name} {arg}"));
    }
}

#[test]
fn dbstop_command_keeps_three_words() {
    let form = parse_matlab_form("dbstop if error").unwrap();
    match &form {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "Command");
            assert_eq!(args.len(), 3, "got {args:?}");
            assert!(args[0].is_symbol("dbstop"));
            assert!(args[1].is_symbol("if"));
            assert!(args[2].is_symbol("error"));
        }
        other => panic!("expected Command, got {other:?}"),
    }
    assert_eq!(render_matlab_form(&form), "dbstop if error");
}

#[test]
fn ode45_keeps_all_call_args() {
    let form = parse_matlab_form("ode45(@(t,y)y, [0, 1], 1)").unwrap();
    match &form {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "ode45");
            assert_eq!(args.len(), 3, "must not strip to last arg, got {args:?}");
            assert!(matches!(&args[0], MatlabForm::Call { head, .. } if head == "Function"));
            assert!(matches!(args[1], MatlabForm::List(_)));
            assert!(matches!(args[2], MatlabForm::Atom(MatlabAtom::Number(_))));
        }
        other => panic!("expected ode45 call, got {other:?}"),
    }
    let h = H::new();
    // Unevaluated residual must keep the call shape (was silent → 1).
    let rendered = h.render(h.eval("ode45(@(t,y)y, [0, 1], 1)"));
    assert!(rendered.starts_with("ode45("), "got {rendered}");
    assert_ne!(rendered, "1", "must not collapse to last arg");
}

#[test]
fn meta_commands_lower_to_reject() {
    let h = H::new();
    for input in ["which sin", "profile on", "format long", "dbstop if error"] {
        let form = parse_matlab_form(input).unwrap_or_else(|e| panic!("{input:?}: {e}"));
        assert_eq!(form.head_name(), Some("Command"), "{input:?} => {form:?}");
        let kind = {
            let mut s = h.s.borrow_mut();
            lower_request(&mut s, &form).kind_name().to_string()
        };
        assert!(kind.contains("Reject") || kind.contains("Control"), "{input:?} => {kind}");
    }
}

#[test]
fn methods_call_keeps_string_arg() {
    let form = parse_matlab_form("methods('double')").unwrap();
    match &form {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "methods");
            assert_eq!(args.len(), 1, "must not strip to string alone, got {args:?}");
            assert!(matches!(&args[0], MatlabForm::Atom(MatlabAtom::String(_))));
        }
        other => panic!("expected methods call, got {other:?}"),
    }
    let h = H::new();
    let rendered = h.render(h.eval("methods('double')"));
    assert!(
        rendered.contains("methods") || rendered.contains("Reject") || rendered.contains("ATHENA") || rendered.contains("unsupported"),
        "must not collapse to 'double', got {rendered}"
    );
    assert_ne!(rendered, "'double'");
    assert_ne!(rendered, "double");
}

#[test]
fn global_persistent_declaration_forms() {
    let global = parse_matlab_form("global x y").unwrap();
    assert_eq!(
        global,
        MatlabForm::call("Global", vec![MatlabForm::symbol("x"), MatlabForm::symbol("y")])
    );
    assert_eq!(render_matlab_form(&global), "global x y");

    let persistent = parse_matlab_form("persistent z").unwrap();
    assert_eq!(persistent, MatlabForm::call("Persistent", vec![MatlabForm::symbol("z")]));
    assert_eq!(render_matlab_form(&persistent), "persistent z");

    // Must not silently evaluate to bare `x` / `z`.
    let h = H::new();
    let g_req = {
        let mut s = h.s.borrow_mut();
        lower_request(&mut s, &global).kind_name().to_string()
    };
    assert!(g_req.contains("Reject") || g_req.contains("Control"), "got {g_req}");
}

#[test]
fn whitespace_juxtaposed_non_command_still_rejected() {
    // Adjacent assignments with only spaces (not command syntax) still error.
    let err = parse_matlab_form("x=1 y=2").expect_err("juxta");
    assert!(
        err.to_string().contains("juxtaposed"),
        "got {err}"
    );
    // Semicolon / comma statement separators remain CompoundExpression.
    let form = parse_matlab_form("a; b").unwrap();
    assert_eq!(form.head_name(), Some("CompoundExpression"));
    let form = parse_matlab_form("a, b").unwrap();
    assert_eq!(form.head_name(), Some("CompoundExpression"));
}

#[test]
fn parfor_and_spmd_keep_typed_forms_and_reject() {
    let parfor = parse_matlab_form("parfor i=1:2, i, end").unwrap();
    assert_eq!(parfor.head_name(), Some("Parfor"));
    assert_eq!(render_matlab_form(&parfor), "parfor i=1:2, i, end");

    let spmd = parse_matlab_form("spmd, 1, end").unwrap();
    assert_eq!(spmd.head_name(), Some("Spmd"));
    assert_eq!(render_matlab_form(&spmd), "spmd, 1, end");

    let mut s = Session::new();
    assert_eq!(lower_request(&mut s, &parfor).kind_name(), "Control");
    assert_eq!(lower_request(&mut s, &spmd).kind_name(), "Control");
}

#[test]
fn cell_brace_literal_keeps_cell_form() {
    let form = parse_matlab_form("{1, 2}").unwrap();
    assert_eq!(form.head_name(), Some("Cell"));
    assert_eq!(render_matlab_form(&form), "{1, 2}");
}

#[test]
fn cell_brace_in_call_args_stays_one_argument() {
    let form = parse_matlab_form("cellfun(@numel, {1, 2})").unwrap();
    match form {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "cellfun");
            assert_eq!(args.len(), 2, "got {args:?}");
            assert_eq!(args[1].head_name(), Some("Cell"));
            assert_eq!(render_matlab_form(&args[1]), "{1, 2}");
        }
        other => panic!("expected cellfun call, got {other:?}"),
    }
    let form = parse_matlab_form("iscell({1})").unwrap();
    match form {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "iscell");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].head_name(), Some("Cell"));
        }
        other => panic!("expected iscell call, got {other:?}"),
    }
}

#[test]
fn ieee_edge_forms_use_nan_and_matlab_zero_pow_zero() {
    let h = H::new();
    assert_eq!(h.render(h.eval("0/0")), "NaN");
    assert_eq!(h.render(h.eval("Inf - Inf")), "NaN");
    assert_eq!(h.render(h.eval("0^0")), "1");
    // Convention applies after binding, not only to literal Form zeros.
    assert_eq!(h.render(h.eval("x = 0; x^0")), "1");
    assert_eq!(h.render(h.eval("0.^0")), "1");
}

#[test]
fn parse_matlab_form_if_without_session() {
    let form = parse_matlab_form("if true, 1, else, 2, end").unwrap();
    assert_eq!(form.head_name(), Some("If"));
}

#[test]
fn member_access_keeps_package_path() {
    let form = parse_matlab_form("containers.Map").unwrap();
    assert_eq!(
        form,
        MatlabForm::call(
            "Member",
            vec![MatlabForm::symbol("containers"), MatlabForm::symbol("Map")]
        )
    );
    assert_eq!(render_matlab_form(&form), "containers.Map");

    let call = parse_matlab_form("containers.Map('a', 1)").unwrap();
    match call {
        MatlabForm::Call { head, args } => {
            assert_eq!(head, "Application");
            assert_eq!(args.len(), 3);
            assert_eq!(args[0].head_name(), Some("Member"));
            assert_eq!(render_matlab_form(&parse_matlab_form("containers.Map('a', 1)").unwrap()), "containers.Map('a', 1)");
        }
        other => panic!("expected Application(Member, …), got {other:?}"),
    }

    let mut s = Session::new();
    assert_eq!(
        lower_request(&mut s, &parse_matlab_form("containers.Map").unwrap()).kind_name(),
        "Control"
    );
    assert_eq!(
        lower_request(&mut s, &parse_matlab_form("py.list([1, 2])").unwrap()).kind_name(),
        "Control"
    );
}
