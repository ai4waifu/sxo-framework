import { feature } from '@sxo/harness';

export const linearAlgebraFeatures = [
    feature('LinearSolve', 'linear_algebra')
        .partial('unique projects as a matrix. Inconsistent and Infinite render as LinearSolve[disposition]')
        .pure()
        .eval('linearsolve.2x2', 'LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]', '{{-4}, {9/2}}')
        .eval('linearsolve.inconsistent', 'LinearSolve[{{1, 2}, {2, 4}}, {{1}, {0}}]', 'LinearSolve[Inconsistent]')
        .eval('linearsolve.infinite', 'LinearSolve[{{1, 2}, {2, 4}}, {{2}, {4}}]', 'LinearSolve[Infinite, 1]')
        .gap('linearsolve.affine', 'LinearSolve[{{1, 2}, {2, 4}}, {{2}, {4}}]', {
            expected: 'affine solution space',
            notes: 'Infinite residual names free_vars but does not publish a parametric family',
        })
        .done(),
    feature('Det', 'linear_algebra')
        .partial('typed numeric nested List determinant only')
        .pure()
        .eval('det.2x2', 'Det[{{1, 2}, {3, 4}}]', '-2')
        .done(),
    feature('Inverse', 'linear_algebra')
        .partial('invertible matrices project. Singular stays Inverse[Singular], not a matrix')
        .pure()
        .eval('inverse.eye', 'Inverse[{{1, 0}, {0, 1}}]', '{{1, 0}, {0, 1}}')
        .eval('inverse.singular', 'Inverse[{{1, 2}, {2, 4}}]', 'Inverse[Singular]')
        .done(),
    feature('Transpose', 'linear_algebra')
        .partial('typed numeric nested List transpose. Ragged stays residual')
        .pure()
        .eval('transpose.2x2', 'Transpose[{{1, 2}, {3, 4}}]', '{{1, 3}, {2, 4}}')
        .eval('transpose.ragged', 'Transpose[{{1, 2}, {3}}]', 'Transpose[{{1, 2}, {3}}]')
        .done(),
    feature('ConjugateTranspose', 'linear_algebra')
        .partial('ComplexExact parent conjugates then transposes. Machine complex parent still open')
        .pure()
        .eval('ctranspose.real', 'ConjugateTranspose[{{1, 2}, {3, 4}}]', '{{1, 3}, {2, 4}}')
        .eval('ctranspose.complex', 'ConjugateTranspose[{{1 + I}}]', '{{1 - I}}')
        .done(),
    feature('Dot', 'linear_algebra')
        .partial('typed numeric nested List matrix-vector and matrix-matrix dot only')
        .pure()
        .eval('dot.mv', 'Dot[{{1, 2}, {3, 4}}, {1, 1}]', '{3, 7}')
        .done(),
    feature('RowReduce', 'linear_algebra')
        .partial('typed numeric nested List RREF only')
        .pure()
        .eval('rowreduce.basic', 'RowReduce[{{1, 2}, {3, 4}}]', '{{1, 0}, {0, 1}}')
        .done(),
    feature('IdentityMatrix', 'linear_algebra')
        .partial('exact integer square `IdentityMatrix[n]` only')
        .pure()
        .eval('idmat.2', 'IdentityMatrix[2]', '{{1, 0}, {0, 1}}')
        .done(),
    feature('Dimensions', 'linear_algebra')
        .partial('typed numeric nested List shape only')
        .pure()
        .eval('dims.2x2', 'Dimensions[{{1, 2}, {3, 4}}]', '{2, 2}')
        .done(),
    feature('MatrixRank', 'linear_algebra')
        .partial('typed numeric nested List rank only')
        .pure()
        .eval('matrixrank.rank1', 'MatrixRank[{{1, 2}, {2, 4}}]', '1')
        .done(),
    feature('Eigenvalues', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('eigenvalues.diag', 'Eigenvalues[{{1, 0}, {0, 2}}]', { expected: '{2, 1}' })
        .done(),
    feature('DiagonalMatrix', 'linear_algebra')
        .partial('vector diagonal constructor with exact complex entries OK')
        .pure()
        .eval('diagmat.12', 'DiagonalMatrix[{1, 2}]', '{{1, 0}, {0, 2}}')
        .eval('diagmat.complex', 'DiagonalMatrix[{1 + I, 2}]', '{{1 + I, 0}, {0, 2}}')
        .done(),
    feature('Tr', 'linear_algebra').partial('typed numeric nested List trace only').pure().eval('tr.2x2', 'Tr[{{1, 2}, {3, 4}}]', '5').done(),
    feature('Norm', 'linear_algebra')
        .partial('vector norm when sum of squares is a perfect square only')
        .pure()
        .eval('norm.34', 'Norm[{3, 4}]', '5')
        .done(),
    feature('Cross', 'linear_algebra')
        .partial('typed numeric 3-vectors only')
        .pure()
        .eval('cross.ijk', 'Cross[{1, 0, 0}, {0, 1, 0}]', '{0, 0, 1}')
        .done(),
    feature('Eigenvectors', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('eigenvectors.diag', 'Eigenvectors[{{1, 0}, {0, 2}}]', { expected: '{{0, 1}, {1, 0}}' })
        .done(),
    feature('NullSpace', 'linear_algebra')
        .partial('typed numeric nested List null space row basis only')
        .pure()
        .eval('nullspace.rank1', 'NullSpace[{{1, 2}, {2, 4}}]', '{{-2, 1}}')
        .done(),
    feature('LowerTriangularize', 'linear_algebra')
        .partial('typed numeric nested List lower triangle only')
        .pure()
        .eval('lower.2x2', 'LowerTriangularize[{{1, 2}, {3, 4}}]', '{{1, 0}, {3, 4}}')
        .done(),
    feature('UpperTriangularize', 'linear_algebra')
        .partial('typed numeric nested List upper triangle only')
        .pure()
        .eval('upper.2x2', 'UpperTriangularize[{{1, 2}, {3, 4}}]', '{{1, 2}, {0, 4}}')
        .eval('upper.complex', 'UpperTriangularize[{{1 + I, 2}, {3, 4 - I}}]', '{{1 + I, 2}, {0, 4 - I}}')
        .done(),
    feature('KroneckerProduct', 'linear_algebra')
        .partial('typed numeric vector Kronecker only. Matrix blocks stay open')
        .pure()
        .eval('kron.vecs', 'KroneckerProduct[{1, 2}, {3, 4}]', '{3, 4, 6, 8}')
        .done(),
    feature('MatrixExp', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('matrixexp.rot', 'MatrixExp[{{0, 1}, {-1, 0}}]', { expected: '...' })
        .done(),
    feature('Normalize', 'linear_algebra').unsupported().pure().gap('normalize.34', 'Normalize[{3, 4}]', { expected: '{3/5, 4/5}' }).done(),
];
