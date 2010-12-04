import { feature } from '@sxo/harness';

export const linearAlgebraFeatures = [
    feature('det', 'linear_algebra').supported().pure().eval('det.2x2', 'det([1, 2; 3, 4])', '-2').done(),
    feature('inv', 'linear_algebra')
        .partial('invertible matrices project. Singular stays inv(Singular), not a matrix')
        .pure()
        .eval('inv.diag', 'inv([1, 0; 0, 2])', '[1, 0; 0, 1/2]')
        .eval('inv.singular', 'inv([1, 2; 2, 4])', 'inv(Singular)')
        .done(),
    feature('rank', 'linear_algebra')
        .supported()
        .pure()
        .notes('rank → MatrixRank Goal → LinearAlgebraRequest::Rank')
        .eval('rank.def', 'rank([1, 2; 2, 4])', '1')
        .done(),
    feature('eig', 'linear_algebra').unsupported().pure().gap('eig.sym', 'eig([1, 2; 2, 1])', { expected: '[3; -1]' }).done(),
    feature('trace', 'linear_algebra')
        .supported()
        .pure()
        .notes('trace → Tr Goal → LinearAlgebraRequest::Trace')
        .eval('trace.2x2', 'trace([1, 2; 3, 4])', '5')
        .done(),
    feature('norm', 'linear_algebra')
        .supported()
        .pure()
        .notes('norm → Norm Goal (exact Euclidean perfect square)')
        .eval('norm.34', 'norm([3, 4])', '5')
        .done(),
    feature('dot', 'linear_algebra')
        .supported()
        .pure()
        .notes('dot → Dot Goal; flat vectors oriented for inner product')
        .eval('dot.2', 'dot([1, 2], [3, 4])', '11')
        .done(),
    feature('cross', 'linear_algebra')
        .supported()
        .pure()
        .notes('cross → Cross Goal on 1×3 / 3×1 vectors')
        .eval('cross.ijk', 'cross([1, 0, 0], [0, 1, 0])', '[0, 0, 1]')
        .done(),
    feature('cond', 'linear_algebra')
        .supported()
        .pure()
        .notes('cond → ConditionNumber Goal; LU pivot-ratio estimate (Singular → Inf)')
        .eval('cond.eye', 'cond([2, 0; 0, 2])', '1')
        .done(),
    feature('null', 'linear_algebra')
        .supported()
        .pure()
        .notes('null → NullSpace Goal with column_basis (Living 16)')
        .eval('null.rank1', 'null([1, 2; 2, 4])', '[-2; 1]')
        .done(),
    feature('diag', 'linear_algebra')
        .supported()
        .pure()
        .notes('diag(v) → DiagonalMatrix Semantic / Form constructor')
        .eval('diag.vec', 'diag([1, 2])', '[1, 0; 0, 2]')
        .done(),
    feature('pinv', 'linear_algebra').unsupported().pure().gap('pinv.2x2', 'pinv([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('svd', 'linear_algebra').unsupported().pure().gap('svd.2x2', 'svd([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('kron', 'linear_algebra')
        .supported()
        .pure()
        .notes('kron → KroneckerProduct Goal → LinearAlgebraRequest::Kronecker')
        .eval('kron.basic', 'kron([1, 2], [3, 4])', '[3, 4, 6, 8]')
        .done(),
    feature('qr', 'linear_algebra').unsupported().pure().gap('qr.2x2', 'qr([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('lu', 'linear_algebra').unsupported().pure().gap('lu.2x2', 'lu([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('chol', 'linear_algebra').unsupported().pure().gap('chol.spd', 'chol([2, 1; 1, 2])', { expected: '...' }).done(),
    feature('expm', 'linear_algebra').unsupported().pure().gap('expm.rot', 'expm([0, 1; -1, 0])', { expected: '...' }).done(),
    feature('rref', 'linear_algebra')
        .supported()
        .pure()
        .notes('rref → RowReduce Goal → LinearAlgebraRequest::Rref')
        .eval('rref.basic', 'rref([1, 2; 2, 4])', '[1, 2; 0, 0]')
        .done(),
    feature('pcg', 'linear_algebra').unsupported().pure().gap('pcg.eye', 'pcg(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('minres', 'linear_algebra').unsupported().pure().gap('minres.eye', 'minres(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('cgs', 'linear_algebra').unsupported().pure().gap('cgs.eye', 'cgs(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('lsqr', 'linear_algebra').unsupported().pure().gap('lsqr.eye', 'lsqr(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('svd_econ', 'linear_algebra').unsupported().pure().gap('svd.econ', 'svd(magic(3), "econ")', { expected: '...' }).done(),
];
