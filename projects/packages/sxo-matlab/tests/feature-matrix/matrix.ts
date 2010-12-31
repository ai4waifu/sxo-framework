import { feature } from '@sxo/harness';

export const matrixFeatures = [
    feature('matrix', 'matrix')
        .partial('literal nested List + constructors via MatrixValue bridge; MatrixId IR pending')
        .pure()
        .eval('matrix.literal', '[1, 2; 3, 4]', '[1, 2; 3, 4]')
        .roundtrip('matrix.roundtrip', '[1 2; 3 4]', '[1, 2; 3, 4]')
        .done(),
    feature('transpose', 'matrix')
        .supported()
        .pure()
        .notes("`.'` Transpose: 2-D via MatrixValue Goal; row/column vectors keep Term reshape")
        .eval('transpose.row', "[1, 2].'", '[1; 2]')
        .eval('transpose.col', "[1; 2].'", '[1, 2]')
        .eval('transpose.mat', "[1, 2; 3, 4].'", '[1, 3; 2, 4]')
        .done(),
    feature('ctranspose', 'matrix')
        .supported()
        .pure()
        .notes('exact ComplexExact conjugates. MATLAB renders 1×1 matrix results as scalars')
        .eval('ctranspose.real', "[1, 2; 3, 4]'", '[1, 3; 2, 4]')
        .eval('ctranspose.complex', "[1+2i, 3; 4, 5]'", '[1 - 2*i, 4; 3, 5]')
        .eval('ctranspose.basic', "(1+1i)'", '1 - i')
        .eval('ctranspose.one_by_one', "[1+1i]'", '1 - i')
        .done(),
    feature('eye', 'matrix').supported().pure().eval('eye.2', 'eye(2)', '[1, 0; 0, 1]').done(),
    feature('zeros', 'matrix').supported().pure().eval('zeros.23', 'zeros(2, 3)', '[0, 0, 0; 0, 0, 0]').done(),
    feature('ones', 'matrix').supported().pure().eval('ones.2', 'ones(2)', '[1, 1; 1, 1]').done(),
    feature('size', 'matrix').supported().pure().eval('size.2x2', 'size([1, 2; 3, 4])', '[2, 2]').done(),
    feature('length', 'matrix').supported().pure().eval('length.vec', 'length([1, 2, 3])', '3').done(),
    feature('sum', 'matrix')
        .supported()
        .pure()
        .notes('default and dim=1 column sums; dim=2 row sums as a column vector; exact complex parent OK')
        .eval('sum.vec', 'sum([1, 2, 3])', '6')
        .eval('sum.matrix', 'sum([1, 2; 3, 4])', '[4, 6]')
        .eval('sum.axis', 'sum([1, 2; 3, 4], 2)', '[3; 7]')
        .eval('sum.complex', 'sum([1+i, 2; 3, 4-i], 2)', '[3 + i; 7 - i]')
        .done(),
    feature('prod', 'matrix')
        .supported()
        .pure()
        .notes('default and dim=1 column products; dim=2 row products as a column vector; exact complex parent OK')
        .eval('prod.vec', 'prod([2, 3, 4])', '24')
        .eval('prod.matrix', 'prod([1, 2; 3, 4])', '[3, 8]')
        .eval('prod.axis', 'prod([1, 2; 3, 4], 2)', '[2; 12]')
        .eval('prod.complex', 'prod([1+i, 2; 3, 4-i], 2)', '[2 + 2*i; 12 - 3*i]')
        .done(),
    feature('max', 'matrix').unsupported().pure().gap('max.vec', 'max([1, 3, 2])', { expected: '3' }).done(),
    feature('linspace', 'matrix').unsupported().pure().gap('linspace.3', 'linspace(0, 1, 3)', { expected: '[0, 0.5, 1]' }).done(),
    feature('reshape', 'matrix').unsupported().pure().gap('reshape.22', 'reshape([1, 2, 3, 4], 2, 2)', { expected: '[1, 3; 2, 4]' }).done(),
    feature('sort', 'matrix').unsupported().pure().gap('sort.vec', 'sort([3, 1, 2])', { expected: '[1, 2, 3]' }).done(),
    feature('sparse', 'matrix').unsupported().pure().gap('sparse.diag', 'sparse([1, 0; 0, 2])', { expected: '...' }).done(),
    feature('pascal', 'matrix').unsupported().pure().gap('pascal.3', 'pascal(3)', { expected: '[1,1,1; 1,2,3; 1,3,6]' }).done(),
    feature('magic', 'matrix').unsupported().pure().gap('magic.3', 'magic(3)', { expected: '...' }).done(),
    feature('tril', 'matrix')
        .supported()
        .pure()
        .notes('tril → LowerTriangularize Goal → LinearAlgebraRequest::Tril')
        .eval('tril.2x2', 'tril([1, 2; 3, 4])', '[1, 0; 3, 4]')
        .done(),
    feature('triu', 'matrix')
        .supported()
        .pure()
        .notes('triu → UpperTriangularize Goal → LinearAlgebraRequest::Triu')
        .eval('triu.2x2', 'triu([1, 2; 3, 4])', '[1, 2; 0, 4]')
        .done(),
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
    feature('isdiag', 'matrix')
        .supported()
        .pure()
        .notes('isdiag → IsDiagonal Goal → 0/1 Dot-surface scalar')
        .eval('isdiag.eye', 'isdiag(eye(3))', '1')
        .done(),
    feature('issymmetric', 'matrix')
        .supported()
        .pure()
        .notes('issymmetric → IsSymmetric Goal → 0/1 Dot-surface scalar')
        .eval('issymmetric.eye', 'issymmetric(eye(3))', '1')
        .done(),
    feature('istril', 'matrix')
        .supported()
        .pure()
        .notes('istril → IsTriangular(lower) Goal; nested tril(…) not MatrixOperand at lower time')
        .eval('istril.lower', 'istril([1, 0; 3, 4])', '1')
        .done(),
    feature('istriu', 'matrix')
        .supported()
        .pure()
        .notes('istriu → IsTriangular(upper) Goal')
        .eval('istriu.upper', 'istriu([1, 2; 0, 4])', '1')
        .done(),
    feature('spalloc', 'matrix').unsupported().pure().gap('spalloc.332', 'spalloc(3, 3, 2)', { expected: '...' }).done(),
    feature('sparse_ijv', 'matrix').unsupported().pure().gap('sparse.ijv', 'sparse(1, 2, 3, 4, 4)', { expected: '...' }).done(),
    feature('full_speye', 'matrix').unsupported().pure().gap('full.speye2', 'full(speye(2))', { expected: '[1, 0; 0, 1]' }).done(),
    feature('spones', 'matrix').unsupported().pure().gap('spones.eye', 'spones(speye(2))', { expected: '...' }).done(),
    feature('spfun', 'matrix')
        .unsupported('parse/eval keep `@sqrt` and `speye(2)` args; spfun sparse map runtime still open')
        .pure()
        .gap('spfun.sqrt', 'spfun(@sqrt, speye(2))', {
            expected: '...',
            notes: 'Form/render: spfun(@Sqrt, speye(2)); was silent strip of @ and/or speye(2)→speye',
        })
        .done(),
];
