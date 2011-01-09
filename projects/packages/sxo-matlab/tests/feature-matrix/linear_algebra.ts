import { feature } from '@sxo/harness';

export const linearAlgebraFeatures = [
    feature('det', 'linear_algebra')
        .partial('typed numeric MatrixValue determinant only')
        .pure()
        .eval('det.2x2', 'det([1, 2; 3, 4])', '-2')
        .done(),
    feature('inv', 'linear_algebra')
        .partial('invertible matrices project. Singular stays inv(Singular), not a matrix')
        .pure()
        .eval('inv.diag', 'inv([1, 0; 0, 2])', '[1, 0; 0, 1/2]')
        .eval('inv.singular', 'inv([1, 2; 2, 4])', 'inv(Singular)')
        .done(),
    feature('rank', 'linear_algebra')
        .partial('typed numeric MatrixValue rank only')
        .pure()
        .eval('rank.def', 'rank([1, 2; 2, 4])', '1')
        .done(),
    feature('eig', 'linear_algebra').unsupported().pure().gap('eig.sym', 'eig([1, 2; 2, 1])', { expected: '[3; -1]' }).done(),
    feature('trace', 'linear_algebra')
        .partial('typed numeric MatrixValue trace only')
        .pure()
        .eval('trace.2x2', 'trace([1, 2; 3, 4])', '5')
        .done(),
    feature('norm', 'linear_algebra')
        .partial('vector norm when sum of squares is a perfect square only')
        .pure()
        .eval('norm.34', 'norm([3, 4])', '5')
        .done(),
    feature('dot', 'linear_algebra')
        .partial('flat numeric vectors for inner product only')
        .pure()
        .eval('dot.2', 'dot([1, 2], [3, 4])', '11')
        .done(),
    feature('cross', 'linear_algebra')
        .partial('1×3 / 3×1 numeric vectors only')
        .pure()
        .eval('cross.ijk', 'cross([1, 0, 0], [0, 1, 0])', '[0, 0, 1]')
        .done(),
    feature('cond', 'linear_algebra')
        .partial('finite estimates project. Singular renders inf, not a condition-number object')
        .pure()
        .eval('cond.eye', 'cond([2, 0; 0, 2])', '1')
        .eval('cond.singular', 'cond([1, 2; 2, 4])', 'inf')
        .done(),
    feature('null', 'linear_algebra')
        .partial('typed numeric MatrixValue null space column basis only')
        .pure()
        .eval('null.rank1', 'null([1, 2; 2, 4])', '[-2; 1]')
        .done(),
    feature('diag', 'linear_algebra')
        .partial('vector diagonal constructor with exact complex entries OK')
        .pure()
        .eval('diag.vec', 'diag([1, 2])', '[1, 0; 0, 2]')
        .eval('diag.complex', 'diag([1+i, 2])', '[1 + i, 0; 0, 2]')
        .done(),
    feature('pinv', 'linear_algebra').unsupported().pure().gap('pinv.2x2', 'pinv([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('svd', 'linear_algebra').unsupported().pure().gap('svd.2x2', 'svd([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('kron', 'linear_algebra')
        .partial('typed numeric vector Kronecker only. Matrix blocks stay open')
        .pure()
        .eval('kron.basic', 'kron([1, 2], [3, 4])', '[3, 4, 6, 8]')
        .done(),
    feature('qr', 'linear_algebra').unsupported().pure().gap('qr.2x2', 'qr([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('lu', 'linear_algebra').unsupported().pure().gap('lu.2x2', 'lu([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('chol', 'linear_algebra').unsupported().pure().gap('chol.spd', 'chol([2, 1; 1, 2])', { expected: '...' }).done(),
    feature('expm', 'linear_algebra').unsupported().pure().gap('expm.rot', 'expm([0, 1; -1, 0])', { expected: '...' }).done(),
    feature('rref', 'linear_algebra')
        .partial('typed numeric MatrixValue RREF only')
        .pure()
        .eval('rref.basic', 'rref([1, 2; 2, 4])', '[1, 2; 0, 0]')
        .done(),
    feature('pcg', 'linear_algebra').unsupported().pure().gap('pcg.eye', 'pcg(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('minres', 'linear_algebra').unsupported().pure().gap('minres.eye', 'minres(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('cgs', 'linear_algebra').unsupported().pure().gap('cgs.eye', 'cgs(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('lsqr', 'linear_algebra').unsupported().pure().gap('lsqr.eye', 'lsqr(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('svd_econ', 'linear_algebra').unsupported().pure().gap('svd.econ', 'svd(magic(3), "econ")', { expected: '...' }).done(),
];
