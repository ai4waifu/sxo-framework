import { feature } from '@sxo/harness';

export const linearAlgebraFeatures = [
    feature('LinearSolve', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Solve')
        .eval('linearsolve.2x2', 'LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]', '{{-4}, {9/2}}')
        .done(),
    feature('Det', 'linear_algebra').supported().pure().eval('det.2x2', 'Det[{{1, 2}, {3, 4}}]', '-2').done(),
    feature('Inverse', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Inverse')
        .eval('inverse.eye', 'Inverse[{{1, 0}, {0, 1}}]', '{{1, 0}, {0, 1}}')
        .done(),
    feature('Transpose', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Transpose')
        .eval('transpose.2x2', 'Transpose[{{1, 2}, {3, 4}}]', '{{1, 3}, {2, 4}}')
        .done(),
    feature('Dot', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Dot')
        .eval('dot.mv', 'Dot[{{1, 2}, {3, 4}}, {1, 1}]', '{3, 7}')
        .done(),
    feature('RowReduce', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Rref')
        .eval('rowreduce.basic', 'RowReduce[{{1, 2}, {3, 4}}]', '{{1, 0}, {0, 1}}')
        .done(),
    feature('IdentityMatrix', 'linear_algebra')
        .supported()
        .pure()
        .notes('surface → SemanticOperator::Eye')
        .eval('idmat.2', 'IdentityMatrix[2]', '{{1, 0}, {0, 1}}')
        .done(),
    feature('Dimensions', 'linear_algebra')
        .supported()
        .pure()
        .notes('surface → SemanticOperator::Size')
        .eval('dims.2x2', 'Dimensions[{{1, 2}, {3, 4}}]', '{2, 2}')
        .done(),
    feature('MatrixRank', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Rank')
        .eval('matrixrank.rank1', 'MatrixRank[{{1, 2}, {2, 4}}]', '1')
        .done(),
    feature('Eigenvalues', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('eigenvalues.diag', 'Eigenvalues[{{1, 0}, {0, 2}}]', { expected: '{2, 1}' })
        .done(),
    feature('DiagonalMatrix', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('diagmat.12', 'DiagonalMatrix[{1, 2}]', { expected: '{{1, 0}, {0, 2}}' })
        .done(),
    feature('Tr', 'linear_algebra')
        .supported()
        .pure()
        .notes('nested List → MatrixValue LinearAlgebraRequest::Trace')
        .eval('tr.2x2', 'Tr[{{1, 2}, {3, 4}}]', '5')
        .done(),
    feature('Norm', 'linear_algebra').unsupported().pure().gap('norm.34', 'Norm[{3, 4}]', { expected: '5' }).done(),
    feature('Cross', 'linear_algebra').unsupported().pure().gap('cross.ijk', 'Cross[{1, 0, 0}, {0, 1, 0}]', { expected: '{0, 0, 1}' }).done(),
    feature('Eigenvectors', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('eigenvectors.diag', 'Eigenvectors[{{1, 0}, {0, 2}}]', { expected: '{{0, 1}, {1, 0}}' })
        .done(),
    feature('NullSpace', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('nullspace.rank1', 'NullSpace[{{1, 2}, {2, 4}}]', { expected: '{{-2}, {1}}' })
        .done(),
    feature('MatrixExp', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('matrixexp.rot', 'MatrixExp[{{0, 1}, {-1, 0}}]', { expected: '...' })
        .done(),
    feature('Normalize', 'linear_algebra').unsupported().pure().gap('normalize.34', 'Normalize[{3, 4}]', { expected: '{3/5, 4/5}' }).done(),
];
