import { feature } from '@sxo/harness';

export const solveFeatures = [
    feature('solve', 'solve')
        .planned('must lower to Athena SolveGoal')
        .pure()
        .gap('solve.linear', 'solve(x-1==0, x)', { expected: '1' })
        .done(),
    feature('mldivide', 'solve')
        .supported()
        .pure()
        .notes('exact numeric A\\b → Solve Goal; inconsistent → []; Infinite keeps particular')
        .eval('mldivide.2x2', '[1,2;3,4] \\ [5;6]', '[-4; 9/2]')
        .eval('mldivide.inconsistent', '[1, 2; 2, 4] \\ [1; 0]', '[]')
        .eval('mldivide.infinite', '[1, 2; 2, 4] \\ [2; 4]', '[2; 0]')
        .done(),
    feature('mrdivide', 'solve')
        .supported()
        .pure()
        .notes('typed matrix A/B → RightSolve; see also arithmetic.mrdivide')
        .eval('mrdivide.row', '[1, 2] / [1, 2; 3, 4]', '[[1, 0]]')
        .done(),
    feature('linsolve', 'solve')
        .supported()
        .pure()
        .notes('linsolve → LinearSolve Form → Athena Solve Goal (same path as mldivide)')
        .eval('linsolve.2x2', 'linsolve([1, 2; 3, 4], [5; 6])', '[-4; 9/2]')
        .done(),
    feature('roots', 'solve').unsupported().pure().gap('roots.quad', 'roots([1, 0, -1])', { expected: '[1; -1]' }).done(),
    feature('fzero', 'solve')
        .unsupported('@ handle stripped; args mangled to fzero(x, -2+x^2, 1)')
        .pure()
        .gap('fzero.sqrt2', 'fzero(@(x)x^2-2, 1)', { expected: '1.4142' })
        .done(),
    feature('fsolve', 'solve')
        .unsupported('@ handle stripped; args mangled like fzero')
        .pure()
        .gap('fsolve.sqrt2', 'fsolve(@(x)x^2-2, 1)', { expected: '1.4142' })
        .done(),
    feature('fminsearch', 'solve')
        .unsupported('@ handle stripped like fzero')
        .pure()
        .gap('fminsearch.x2', 'fminsearch(@(x)x^2, 1)', { expected: '0' })
        .done(),
    feature('lsqcurvefit', 'solve')
        .unsupported('Call Form kept (FunctionHandle + args); no lsqcurvefit solver runtime')
        .pure()
        .gap('lsqcurvefit.strip', 'lsqcurvefit(@(x,xdata)x*xdata, 1, [1, 2], [1, 2])', {
            expected: '1',
            notes: 'must keep full call with @ handle; not silent last-arg strip to [1, 2]',
        })
        .done(),
    feature('fminbnd', 'solve').unsupported('@ handle stripped').pure().gap('fminbnd.x2', 'fminbnd(@(x)x^2, -1, 1)', { expected: '0' }).done(),
];
