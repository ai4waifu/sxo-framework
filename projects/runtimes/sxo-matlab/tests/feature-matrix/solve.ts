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
        .notes('exact numeric nested-list A\\b; symbolic stays unevaluated')
        .eval('mldivide.2x2', '[1,2;3,4] \\ [5;6]', '[-4; 9/2]')
        .done(),
    feature('linsolve', 'solve')
        .unsupported('linsolve → LinearSolve; same exact bridge as mldivide')
        .pure()
        .gap('linsolve.2x2', 'linsolve([1, 2; 3, 4], [5; 6])', { expected: '[-4; 9/2]', notes: 'currently linsolve([1, 2; 3, 4], [5; 6])' })
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
        .unsupported('SILENT WRONG: @ stripped and result collapses to last arg [1,2]')
        .pure()
        .gap('lsqcurvefit.strip', 'lsqcurvefit(@(x,xdata)x*xdata, 1, [1, 2], [1, 2])', { expected: '1', notes: 'currently returns [1, 2]' })
        .done(),
    feature('fminbnd', 'solve').unsupported('@ handle stripped').pure().gap('fminbnd.x2', 'fminbnd(@(x)x^2, -1, 1)', { expected: '0' }).done(),
];
