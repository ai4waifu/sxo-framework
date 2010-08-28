import { feature } from '@sxo/harness';

export const solveFeatures = [
    feature('Solve', 'solve')
        .supported()
        .pure()
        .notes('univariate polynomial equations lower to Athena SolveGoal')
        .eval('solve.quad', 'Solve[x^2 == 1, x]', '{{x -> -1}, {x -> 1}}')
        .done(),
    feature('NSolve', 'solve').planned().pure().gap('nsolve.quad', 'NSolve[x^2 == 1, x]', { expected: '{{x -> -1.}, {x -> 1.}}' }).done(),
    feature('Reduce', 'solve').planned().pure().gap('reduce.basic', 'Reduce[x^2 > 0, x]', { expected: 'x < 0 || x > 0' }).done(),
    feature('FindRoot', 'solve').planned().pure().gap('findroot.basic', 'FindRoot[x^2 - 2, {x, 1}]', { expected: '{x -> 1.41421}' }).done(),
    feature('DSolve', 'solve')
        .planned("y' sugar parses; DSolve ODE solver runtime still open")
        .pure()
        .gap('dsolve.basic', "DSolve[y'[x] == y[x], y[x], x]", { expected: '{{y[x] -> E^x C[1]}}' })
        .done(),
    feature('Eliminate', 'solve')
        .planned()
        .pure()
        .gap('eliminate.basic', 'Eliminate[{x + y == 1, x - y == 0}, y]', { expected: 'x == 1/2' })
        .done(),
    feature('FindInstance', 'solve')
        .planned()
        .pure()
        .gap('findinstance.circle', 'FindInstance[x^2 + y^2 == 1, {x, y}]', { expected: '{{x -> 1, y -> 0}}' })
        .done(),
    feature('NDSolve', 'solve')
        .planned("oak parses y'[x] as Derivative; NDSolve runtime still open")
        .pure()
        .gap('ndsolve.exp', "NDSolve[{y'[x] == y[x], y[0] == 1}, y, {x, 0, 1}]", { expected: '...' })
        .done(),
    feature('FindMinimum', 'solve').planned().pure().gap('findminimum.x2', 'FindMinimum[x^2, {x, 1}]', { expected: '{0., {x -> 0.}}' }).done(),
    feature('DSolveValue', 'solve')
        .planned("y' sugar parses as Derivative[1][y][x]; DSolveValue residual (no ODE solver yet)")
        .pure()
        .gap('dsolvevalue.strip', "DSolveValue[y'[x] == y[x], y[x], x]", {
            expected: 'C[1]*Exp[x]',
            notes: "must not collapse to bare x; Form keeps y' / Derivative",
        })
        .done(),
    feature('Maximize', 'solve')
        .planned('Maximize kernel incomplete; unary-minus/Power parse is fixed (`-x^2` keeps sign)')
        .pure()
        .gap('maximize.neg_quad', 'Maximize[-x^2, x]', { expected: '{0, {x -> 0}}' })
        .done(),
    feature('Minimize', 'solve').planned().pure().gap('minimize.quad', 'Minimize[x^2 + 1, x]', { expected: '{1, {x -> 0}}' }).done(),
];
