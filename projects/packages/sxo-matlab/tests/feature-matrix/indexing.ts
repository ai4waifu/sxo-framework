import { feature } from '@sxo/harness';

export const indexingFeatures = [
    feature('colon', 'indexing')
        .partial('integer `1:n` and `a:step:b` row vectors only')
        .pure()
        .eval('colon.range', '1:3', '[1, 2, 3]')
        .eval('colon.step', '1:2:10', '[1, 3, 5, 7, 9]')
        .done(),
    feature('subsref', 'indexing')
        .partial('1-based subsref on literals including linear and slice forms')
        .pure()
        .eval('subsref.vec', '[1, 2, 3](2)', '2')
        .eval('subsref.matrix', '[1, 2; 3, 4](1, 2)', '2')
        .eval('subsref.complex', '[1+i, 2; 3, 4](1, 2)', '2')
        .eval('subsref.linear', '[1, 2; 3, 4](2)', '3')
        .eval('subsref.linear4', '[1, 2; 3, 4](4)', '4')
        .eval('subsref.slice', '[1, 2, 3](1:2)', '[1, 2]')
        .done(),
    feature('end', 'indexing').partial('`end` on literal vectors only').pure().eval('end.index', '[1, 2, 3](end)', '3').done(),
    feature('row_colon', 'indexing')
        .partial('row/col `:` slices on literal matrices only')
        .pure()
        .eval('row.colon', '[1, 2; 3, 4](1,:)', '[1, 2]')
        .eval('col.colon', '[1, 2; 3, 4](:,2)', '[2, 4]')
        .eval('col.colon.complex', '[1+i, 2; 3, 4](:,2)', '[2, 4]')
        .done(),
    feature('logical_index', 'indexing').unsupported().pure().gap('logical.gt', 'A=[1,2,3]; A(A>1)', { expected: '[2, 3]' }).done(),
    feature('colon_all', 'indexing')
        .partial('`A(:)` column-major flatten on literal rectangular matrices')
        .pure()
        .eval('colon.all', '[1, 2; 3, 4](:)', '[1; 3; 2; 4]')
        .eval('colon.all_own', 'A=[1, 2; 3, 4]; A(:)', '[1; 3; 2; 4]')
        .done(),
    feature('end_minus', 'indexing')
        .unsupported('oak error on A(end-1) and A(1:2:end)')
        .pure()
        .gap('end.minus1', 'A=[1, 2, 3]; A(end-1)', { expected: '2' })
        .done(),
    feature('end_slice', 'indexing')
        .unsupported('D(1:end-1) / E(1:end,end) oak error; C(0) stays C(0)')
        .pure()
        .gap('end.slice_minus', 'D=[1, 2, 3]; D(1:end-1)', { expected: '[1, 2]', notes: 'oak error node' })
        .gap('end.slice_2d', 'E=[1, 2; 3, 4]; E(1:end, end)', { expected: '[2; 4]', notes: 'oak error node' })
        .done(),
];
