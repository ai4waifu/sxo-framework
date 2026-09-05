import { feature } from '@sxo/harness';

export const matrixFeatures = [
    feature('matrix', 'matrix')
        .partial('literal nested List + constructors via MatrixValue bridge; MatrixId IR pending')
        .pure()
        .eval('matrix.literal', '[1, 2; 3, 4]', '[1, 2; 3, 4]')
        .roundtrip('matrix.roundtrip', '[1 2; 3 4]', '[1, 2; 3, 4]')
        .done(),
    feature('transpose', 'matrix')
        .unsupported("oak rejects ' transpose literal")
        .pure()
        .gap('transpose.vec', "[1; 2]'", { expected: '[1, 2]' })
        .done(),
    feature('ctranspose', 'matrix')
        .unsupported("depends on complex literal + '")
        .pure()
        .gap('ctranspose.basic', "[1+1i]'", { expected: '1-1i' })
        .done(),
    feature('eye', 'matrix').supported().pure().eval('eye.2', 'eye(2)', '[1, 0; 0, 1]').done(),
    feature('zeros', 'matrix').supported().pure().eval('zeros.23', 'zeros(2, 3)', '[0, 0, 0; 0, 0, 0]').done(),
    feature('ones', 'matrix').supported().pure().eval('ones.2', 'ones(2)', '[1, 1; 1, 1]').done(),
    feature('size', 'matrix').supported().pure().eval('size.2x2', 'size([1, 2; 3, 4])', '[2, 2]').done(),
    feature('length', 'matrix').supported().pure().eval('length.vec', 'length([1, 2, 3])', '3').done(),
    feature('sum', 'matrix')
        .supported()
        .pure()
        .notes('vector → scalar; matrix → column sums')
        .eval('sum.vec', 'sum([1, 2, 3])', '6')
        .eval('sum.matrix', 'sum([1, 2; 3, 4])', '[4, 6]')
        .done(),
    feature('max', 'matrix').unsupported().pure().gap('max.vec', 'max([1, 3, 2])', { expected: '3' }).done(),
    feature('linspace', 'matrix').unsupported().pure().gap('linspace.3', 'linspace(0, 1, 3)', { expected: '[0, 0.5, 1]' }).done(),
    feature('reshape', 'matrix').unsupported().pure().gap('reshape.22', 'reshape([1, 2, 3, 4], 2, 2)', { expected: '[1, 3; 2, 4]' }).done(),
    feature('sort', 'matrix').unsupported().pure().gap('sort.vec', 'sort([3, 1, 2])', { expected: '[1, 2, 3]' }).done(),
    feature('sparse', 'matrix').unsupported().pure().gap('sparse.diag', 'sparse([1, 0; 0, 2])', { expected: '...' }).done(),
    feature('pascal', 'matrix').unsupported().pure().gap('pascal.3', 'pascal(3)', { expected: '[1,1,1; 1,2,3; 1,3,6]' }).done(),
    feature('magic', 'matrix').unsupported().pure().gap('magic.3', 'magic(3)', { expected: '...' }).done(),
    feature('tril', 'matrix').unsupported().pure().gap('tril.2x2', 'tril([1, 2; 3, 4])', { expected: '[1, 0; 3, 4]' }).done(),
    feature('hilb', 'matrix').unsupported().pure().gap('hilb.3', 'hilb(3)', { expected: '...' }).done(),
    feature('blkdiag', 'matrix').unsupported().pure().gap('blkdiag.eye3', 'blkdiag(eye(2), 3)', { expected: '...' }).done(),
    feature('numel', 'matrix').unsupported().pure().gap('numel.empty', 'numel([])', { expected: '0' }).done(),
    feature('nan_matrix', 'matrix').unsupported().pure().gap('nan.2', 'nan(2)', { expected: '[NaN, NaN; NaN, NaN]' }).done(),
    feature('inf_matrix', 'matrix').unsupported().pure().gap('inf.2', 'inf(2)', { expected: '...' }).done(),
    feature('true_matrix', 'matrix').unsupported().pure().gap('true.23', 'true(2, 3)', { expected: '...' }).done(),
    feature('speye', 'matrix').unsupported().pure().gap('speye.3', 'speye(3)', { expected: '...' }).done(),
    feature('nnz', 'matrix').unsupported().pure().gap('nnz.speye2', 'nnz(speye(2))', { expected: '2' }).done(),
    feature('logspace', 'matrix').unsupported().pure().gap('logspace.3', 'logspace(0, 2, 3)', { expected: '[1, 10, 100]' }).done(),
    feature('zeros_empty', 'matrix').unsupported().pure().gap('zeros.0x5', 'zeros(0, 5)', { expected: 'zeros(0,5)' }).done(),
    feature('ones_empty', 'matrix').unsupported().pure().gap('ones.5x0', 'ones(5, 0)', { expected: 'ones(5,0)' }).done(),
    feature('eye_empty', 'matrix').unsupported().pure().gap('eye.0', 'eye(0)', { expected: '[]' }).done(),
    feature('isdiag', 'matrix').unsupported().pure().gap('isdiag.eye', 'isdiag(eye(3))', { expected: '1' }).done(),
    feature('issymmetric', 'matrix').unsupported().pure().gap('issymmetric.eye', 'issymmetric(eye(3))', { expected: '1' }).done(),
    feature('istril', 'matrix').unsupported().pure().gap('istril.tril', 'istril(tril(ones(3)))', { expected: '1' }).done(),
    feature('spalloc', 'matrix').unsupported().pure().gap('spalloc.332', 'spalloc(3, 3, 2)', { expected: '...' }).done(),
    feature('sparse_ijv', 'matrix').unsupported().pure().gap('sparse.ijv', 'sparse(1, 2, 3, 4, 4)', { expected: '...' }).done(),
    feature('full_speye', 'matrix').unsupported().pure().gap('full.speye2', 'full(speye(2))', { expected: '[1, 0; 0, 1]' }).done(),
    feature('spones', 'matrix').unsupported().pure().gap('spones.eye', 'spones(speye(2))', { expected: '...' }).done(),
    feature('spfun', 'matrix')
        .unsupported('SILENT WRONG: @ stripped — spfun(@sqrt,speye(2)) → spfun(sqrt, speye(2))')
        .pure()
        .gap('spfun.sqrt', 'spfun(@sqrt, speye(2))', { expected: '...', notes: 'currently spfun(sqrt, speye(2))' })
        .done(),
];
