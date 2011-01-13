import { feature } from '@sxo/harness';

export const arrayFeatures = [
    feature('diff_vector', 'array')
        .unsupported('vector difference op distinct from symbolic diff(f,x)')
        .pure()
        .gap('diffvec.3', 'diff([1, 4, 9])', { expected: '[3, 5]' })
        .done(),
    feature('cumsum', 'array')
        .partial('row-vector `cumsum` on tested forms. Exact complex parent OK')
        .pure()
        .eval('cumsum.3', 'cumsum([1, 2, 3])', '[1, 3, 6]')
        .eval('cumsum.complex', 'cumsum([1+i, 2, 3])', '[1 + i, 3 + i, 6 + i]')
        .done(),
    feature('bsxfun', 'array')
        .unsupported('parse/eval keep `@plus` FunctionHandle; bsxfun broadcast runtime still open')
        .pure()
        .gap('bsxfun.plus', 'bsxfun(@plus, [1, 2], [3; 4])', {
            expected: '[4, 5; 5, 6]',
            notes: 'Form/render: bsxfun(@plus, [1, 2], [3; 4]); was silent strip of @',
        })
        .done(),
    feature('arrayfun', 'array')
        .unsupported('parse/eval keep `@sin` FunctionHandle; arrayfun map runtime still open')
        .pure()
        .gap('arrayfun.sin', 'arrayfun(@sin, [0, pi/2])', {
            expected: '[0, 1]',
            notes: 'Form keeps FunctionHandle; was silent strip to bare sin',
        })
        .done(),
    feature('cellfun', 'array')
        .unsupported('parse keeps cell arg via oak CellArray; cellfun execution still open')
        .pure()
        .gap('cellfun.numel', 'cellfun(@numel, {1, 2})', {
            expected: '[1, 1]',
            notes: 'Form is cellfun(@numel, Cell({1,2})); runtime pending',
        })
        .done(),
    feature('all', 'array').unsupported().pure().gap('all.true', 'all([1, 1])', { expected: '1' }).done(),
    feature('any', 'array').unsupported().pure().gap('any.mixed', 'any([0, 1])', { expected: '1' }).done(),
    feature('find', 'array').unsupported().pure().gap('find.mask', 'find([0, 1, 0])', { expected: '2' }).done(),
    feature('unique', 'array').unsupported().pure().gap('unique.112', 'unique([1, 1, 2])', { expected: '[1, 2]' }).done(),
    feature('repmat', 'array').unsupported().pure().gap('repmat.21', 'repmat([1, 2], 2, 1)', { expected: '[1, 2; 1, 2]' }).done(),
    feature('intersect', 'array').unsupported().pure().gap('intersect.basic', 'intersect([1, 2, 3], [2, 3, 4])', { expected: '[2, 3]' }).done(),
    feature('union', 'array').unsupported().pure().gap('union.basic', 'union([1, 2], [2, 3])', { expected: '[1, 2, 3]' }).done(),
    feature('setdiff', 'array').unsupported().pure().gap('setdiff.basic', 'setdiff([1, 2, 3], [2])', { expected: '[1, 3]' }).done(),
    feature('ismember', 'array').unsupported().pure().gap('ismember.2', 'ismember([1, 2, 3], 2)', { expected: '[0, 1, 0]' }).done(),
    feature('accumarray', 'array')
        .unsupported()
        .pure()
        .gap('accumarray.basic', 'accumarray([1; 2; 1], [10; 20; 30])', { expected: '[40; 20]' })
        .done(),
];
