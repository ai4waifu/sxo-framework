import { feature } from '@sxo/harness';

export const listFeatures = [
    feature('List', 'list')
        .partial('literal nested list surface and roundtrip only')
        .pure()
        .eval('list.literal', '{1, 2, 3}', '{1, 2, 3}')
        .roundtrip('list.roundtrip', '{1, 2}', '{1, 2}')
        .done(),
    feature('ReplacePart', 'list')
        .partial('Own-mutating StoreIndex for bound symbols. Literal lists return the updated collection')
        .stateful()
        .eval('replacepart.scalar', 'A={1, 2, 3}; ReplacePart[A, 2 -> 9]; A', '{1, 9, 3}')
        .eval('replacepart.coord', 'A={{1, 2}, {3, 4}}; ReplacePart[A, {1, 2} -> 9]; A', '{{1, 9}, {3, 4}}')
        .eval('replacepart.literal', 'ReplacePart[{1, 2, 3}, 2 -> 9]', '{1, 9, 3}')
        .eval('replacepart.empty_grow', 'ReplacePart[{}, 1 -> 1]', '{1}')
        .done(),
    feature('Part', 'list')
        .partial('1-based Part on tested literal forms. `[[0]]` → `{}` per Index contract')
        .pure()
        .eval('part.infix', '{1, 2, 3}[[2]]', '2')
        .eval('part.head', 'Part[{1, 2, 3}, 1]', '1')
        .eval('part.zero', '{1, 2, 3}[[0]]', '{}')
        .eval('part.complex', 'Part[{{1 + I, 2}, {3, 4}}, 1, 2]', '2')
        .done(),
    feature('Range', 'list').partial('`Range[n]` positive integer sequence only').pure().eval('range.3', 'Range[3]', '{1, 2, 3}').done(),
    feature('Map', 'list')
        .partial('head Map keeps exact `Sin[1]`. Slot `Map` folds integers only')
        .pure()
        .eval('map.sin', 'Map[Sin, {0, 1}]', '{0, Sin[1]}')
        .eval('map.slot', 'Map[#^2 &, {1, 2, 3}]', '{1, 4, 9}')
        .done(),
    feature('Table', 'list')
        .partial('single-iterator `Table[i, {i, n}]` contract only')
        .pure()
        .eval('table.basic', 'Table[i, {i, 3}]', '{1, 2, 3}')
        .done(),
    feature('Sum', 'list')
        .partial('single-iterator / list fold via Table only')
        .pure()
        .eval('sum.basic', 'Sum[i, {i, 1, 10}]', '55')
        .done(),
    feature('Product', 'list')
        .partial('iterator Product via Table fold and tested matrix column fold. Deeper specs open')
        .pure()
        .eval('product.basic', 'Product[i, {i, 1, 5}]', '120')
        .eval('product.cols', 'Product[{{1, 2}, {3, 4}}]', '{3, 8}')
        .eval('product.complex_bare', 'Product[{{1 + I, 2}, {3, 4 - I}}]', '{3 + 3*I, 8 - 2*I}')
        .eval('product.complex', 'Product[{{1 + I, 2}, {3, 4 - I}}, {2}]', '{{2 + 2*I}, {12 - 3*I}}')
        .done(),
    feature('Length', 'list').partial('top-level list length on tested forms only').pure().eval('length.3', 'Length[{1, 2, 3}]', '3').done(),
    feature('First', 'list')
        .partial('non-empty List only. Empty → `InvalidIndex`. Exact complex OK')
        .pure()
        .eval('first.ab', 'First[{a, b}]', 'a')
        .eval('first.complex', 'First[{1 + I, 2, 3}]', '1 + I')
        .done(),
    feature('Join', 'list')
        .partial('list concat and matrix row stack. Exact complex parents unify with `ℤ`/`ℚ`')
        .pure()
        .eval('join.basic', 'Join[{1}, {2}]', '{1, 2}')
        .eval('join.complex_mix', 'Join[{{1 + I}}, {{2}}]', '{{1 + I}, {2}}')
        .eval('join.complex_matrix', 'Join[{{1 + I, 2}}, {{3, 4}}]', '{{1 + I, 2}, {3, 4}}')
        .done(),
    feature('Flatten', 'list')
        .partial('row-major flatten on tested nested lists. Exact complex parent OK')
        .pure()
        .eval('flatten.basic', 'Flatten[{{1, 2}, {3}}]', '{1, 2, 3}')
        .eval('flatten.complex', 'Flatten[{{1 + I, 2}, {3, 4 - I}}]', '{1 + I, 2, 3, 4 - I}')
        .done(),
    feature('Apply', 'list').partial('head `Apply` on tested exact lists only').pure().eval('apply.plus', 'Apply[Plus, {1, 2, 3}]', '6').done(),
    feature('Rest', 'list').partial('tail drop on non-empty tested lists only').pure().eval('rest.basic', 'Rest[{1, 2, 3}]', '{2, 3}').done(),
    feature('Most', 'list').partial('drop last on non-empty tested lists only').pure().eval('most.basic', 'Most[{1, 2, 3}]', '{1, 2}').done(),
    feature('Take', 'list')
        .partial('prefix take on exact vectors. Exact complex parent OK')
        .pure()
        .eval('take.2', 'Take[{1, 2, 3, 4}, 2]', '{1, 2}')
        .eval('take.complex', 'Take[{1 + I, 2, 3, 4}, 2]', '{1 + I, 2}')
        .done(),
    feature('Drop', 'list').partial('prefix drop on tested exact lists only').pure().eval('drop.2', 'Drop[{1, 2, 3, 4}, 2]', '{3, 4}').done(),
    feature('Reverse', 'list')
        .partial('vector and matrix reverse on tested forms. Exact complex parent OK')
        .pure()
        .eval('reverse.3', 'Reverse[{1, 2, 3}]', '{3, 2, 1}')
        .eval('reverse.complex', 'Reverse[{1 + I, 2, 3}]', '{3, 2, 1 + I}')
        .done(),
    feature('Sort', 'list')
        .partial('exact-integer ascending contract only. Mixed or non-integer stays residual')
        .pure()
        .eval('sort.3', 'Sort[{3, 1, 2}]', '{1, 2, 3}')
        .done(),
    feature('MemberQ', 'list')
        .partial('structural equality membership only. Not pattern `MemberQ`')
        .pure()
        .eval('memberq.2', 'MemberQ[{1, 2, 3}, 2]', 'True')
        .done(),
    feature('Select', 'list')
        .unsupported('unevaluated Select[list, EvenQ] (no predicate fold yet)')
        .pure()
        .gap('select.evenq', 'Select[{1, 2, 3, 4}, EvenQ]', {
            expected: '{2, 4}',
            notes: 'stays Select[…]; not a silent strip to EvenQ',
        })
        .done(),
    feature('Cases', 'list')
        .partial('CollectMatches via `Blank[Integer]` only. Not full pattern `Cases`')
        .pure()
        .eval('cases.integer', 'Cases[{1, 2, 3}, _Integer]', '{1, 2, 3}')
        .eval('cases.mixed', 'Cases[{1, a, 2}, _Integer]', '{1, 2}')
        .done(),
    feature('Count', 'list')
        .partial('structural equality occurrence count only. Not pattern `Count`')
        .pure()
        .eval('count.1', 'Count[{1, 1, 2}, 1]', '2')
        .done(),
    feature('Partition', 'list')
        .partial('fixed-width exact-integer partitions only')
        .pure()
        .eval('partition.2', 'Partition[{1, 2, 3, 4}, 2]', '{{1, 2}, {3, 4}}')
        .done(),
    feature('Union', 'list')
        .partial('structural dedupe of list args. Exact integers sorted ascending only')
        .pure()
        .eval('union.basic', 'Union[{1, 2}, {2, 3}]', '{1, 2, 3}')
        .done(),
    feature('Intersection', 'list')
        .partial('structural intersection. Exact integers sorted ascending only')
        .pure()
        .eval('intersection.basic', 'Intersection[{1, 2}, {2, 3}]', '{2}')
        .done(),
    feature('FreeQ', 'list')
        .partial('top-level structural non-membership only. Not deep or pattern `FreeQ`')
        .pure()
        .eval('freeq.3', 'FreeQ[{1, 2}, 3]', 'True')
        .done(),
    feature('Position', 'list')
        .partial('top-level structural_eq positions only. Not deep or pattern `Position`')
        .pure()
        .eval('position.1', 'Position[{1, 2, 1}, 1]', '{{1}, {3}}')
        .done(),
    feature('Extract', 'list')
        .partial('1-based extract on tested forms. Exact complex parent OK')
        .pure()
        .eval('extract.2', 'Extract[{1, 2, 3}, 2]', '2')
        .eval('extract.complex', 'Extract[{1 + I, 2, 3}, 2]', '2')
        .done(),
    feature('MapAt', 'list')
        .unsupported('not lowered. Residual MapAt call must not be marked supported')
        .pure()
        .gap('mapat.f2', 'MapAt[f, {1, 2, 3}, 2]', {
            expected: '{1, f[2], 3}',
            notes: 'currently MapAt[f, {1, 2, 3}, 2]',
        })
        .done(),
    feature('PadLeft', 'list')
        .partial('left-pad exact integer `0` or left-truncate. No pad value or level args')
        .pure()
        .eval('padleft.4', 'PadLeft[{1, 2}, 4]', '{0, 0, 1, 2}')
        .eval('padleft.complex', 'PadLeft[{1 + I, 2}, 4]', '{0, 0, 1 + I, 2}')
        .done(),
    feature('Riffle', 'list')
        .partial('top-level zip of two lists with length = min. Not MMA multi-arg or x-spacer')
        .pure()
        .eval('riffle.ab', 'Riffle[{1, 2}, {a, b}]', '{1, a, 2, b}')
        .eval('riffle.complex_mix', 'Riffle[{1, 2}, {I, 3}]', '{1, I, 2, 3}')
        .done(),
    feature('Accumulate', 'list')
        .partial('prefix sums on exact vectors. Exact complex parent OK')
        .pure()
        .eval('accumulate.3', 'Accumulate[{1, 2, 3}]', '{1, 3, 6}')
        .eval('accumulate.complex', 'Accumulate[{1 + I, 2, 3}]', '{1 + I, 3 + I, 6 + I}')
        .done(),
    feature('Differences', 'list')
        .partial('adjacent diffs on exact vectors. Exact complex parent OK')
        .pure()
        .eval('differences.3', 'Differences[{1, 4, 9}]', '{3, 5}')
        .eval('differences.range', 'Differences[Range[4]]', '{1, 1, 1}')
        .eval('differences.complex', 'Differences[{1 + I, 2, 3 - I}]', '{1 - I, 1 - I}')
        .done(),
    feature('Total', 'list')
        .partial('default column fold and tested level specs only. Deeper level specs remain open')
        .pure()
        .eval('total.3', 'Total[{1, 2, 3}]', '6')
        .eval('total.cols', 'Total[{{1, 2}, {3, 4}}]', '{4, 6}')
        .eval('total.complex_bare', 'Total[{{1 + I, 2}, {3, 4 - I}}]', '{4 + I, 6 - I}')
        .eval('total.level', 'Total[{{1, 2}, {3, 4}}, {2}]', '{{3}, {7}}')
        .eval('total.complex', 'Total[{{1 + I, 2}, {3, 4 - I}}, {2}]', '{{3 + I}, {7 - I}}')
        .done(),
    feature('Append', 'list')
        .partial('row-vector scalar extend. Exact complex scalars promote `ℤ`/`ℚ` parents')
        .pure()
        .eval('append.3', 'Append[{1, 2}, 3]', '{1, 2, 3}')
        .eval('append.complex', 'Append[{1, 2}, I]', '{1, 2, I}')
        .done(),
    feature('Prepend', 'list')
        .partial('row-vector scalar prepend. Exact complex scalars promote integer and rational parents')
        .pure()
        .eval('prepend.1', 'Prepend[{2, 3}, 1]', '{1, 2, 3}')
        .eval('prepend.complex', 'Prepend[{1, 2}, I]', '{I, 1, 2}')
        .done(),
    feature('DeleteDuplicates', 'list')
        .partial('structural dedupe on tested exact elements only')
        .pure()
        .eval('deletedup.112', 'DeleteDuplicates[{1, 1, 2}]', '{1, 2}')
        .done(),
    feature('Array', 'list')
        .partial('`Array[f, n]` operator-value head and exact `n` only. No dims or list specs')
        .pure()
        .eval('array.f3', 'Array[f, 3]', '{f[1], f[2], f[3]}')
        .done(),
    feature('ConstantArray', 'list')
        .partial('exact scalar fill including `I` / `1+I` on `ComplexExact` parent')
        .pure()
        .eval('constarray.0', 'ConstantArray[0, 3]', '{0, 0, 0}')
        .eval('constarray.complex', 'ConstantArray[I, 3]', '{I, I, I}')
        .done(),
    feature('DeleteCases', 'list')
        .partial('CollectRejects via `Blank[Integer]` only. Inverse of contracted `Cases`')
        .pure()
        .eval('deletecases.int', 'DeleteCases[{1, a, 2}, _Integer]', '{a}')
        .done(),
    feature('MapIndexed', 'list')
        .partial('MapIndexed applies `f[elem,{i}]`. Multi-slot `#2&` contract only')
        .pure()
        .eval('mapindexed.slot2', 'MapIndexed[#2 &, {a, b}]', '{{1}, {2}}')
        .done(),
    feature('MapThread', 'list')
        .partial('zip columns of `{list1, list2, …}` then apply head or Function')
        .pure()
        .eval('mapthread.f', 'MapThread[f, {{1, 2}, {3, 4}}]', '{f[1, 3], f[2, 4]}')
        .done(),
    feature('Nearest', 'list').unsupported().pure().gap('nearest.3', 'Nearest[{1, 2, 4}, 3]', { expected: '{2, 4}' }).done(),
    feature('Counts', 'list').unsupported().pure().gap('counts.aab', 'Counts[{a, a, b}]', { expected: '<|a -> 2, b -> 1|>' }).done(),
    feature('ListConvolve', 'list').unsupported().pure().gap('listconvolve.basic', 'ListConvolve[{1, 2}, {3, 4}]', { expected: '{11}' }).done(),
    feature('ArrayFlatten', 'list')
        .unsupported()
        .pure()
        .gap('arrayflatten.2x2', 'ArrayFlatten[{{{1, 2}}, {{3, 4}}}]', { expected: '{{1, 2}, {3, 4}}' })
        .done(),
    feature('TensorProduct', 'list')
        .unsupported()
        .pure()
        .gap('tensorproduct.vec', 'TensorProduct[{1, 2}, {3, 4}]', { expected: '{{3, 4}, {6, 8}}' })
        .done(),
    feature('MapAll', 'list').unsupported().pure().gap('mapall.nest', 'MapAll[f, {1, {2}}]', { expected: 'f[{f[1], f[{f[2]}]}]' }).done(),
];
