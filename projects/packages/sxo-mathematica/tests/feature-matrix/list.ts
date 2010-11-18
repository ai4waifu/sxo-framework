import { feature } from '@sxo/harness';

export const listFeatures = [
    feature('List', 'list')
        .supported()
        .pure()
        .eval('list.literal', '{1, 2, 3}', '{1, 2, 3}')
        .roundtrip('list.roundtrip', '{1, 2}', '{1, 2}')
        .done(),
    feature('ReplacePart', 'list')
        .partial('Own-mutating StoreIndex path for scalar Rule; coordinate/matrix Rule and non-mutating return deferred')
        .stateful()
        .eval('replacepart.scalar', 'A={1, 2, 3}; ReplacePart[A, 2 -> 9]; A', '{1, 9, 3}')
        .done(),
    feature('Part', 'list')
        .supported()
        .pure()
        .notes('1-based Part; [[0]] → empty list (kernel Index contract)')
        .eval('part.infix', '{1, 2, 3}[[2]]', '2')
        .eval('part.head', 'Part[{1, 2, 3}, 1]', '1')
        .eval('part.zero', '{1, 2, 3}[[0]]', '{}')
        .done(),
    feature('Range', 'list').supported().pure().notes('Range[n] integer sequence').eval('range.3', 'Range[3]', '{1, 2, 3}').done(),
    feature('Map', 'list')
        .supported()
        .pure()
        .notes('Map keeps exact Sin[1]; Slot Map folds integers')
        .eval('map.sin', 'Map[Sin, {0, 1}]', '{0, Sin[1]}')
        .eval('map.slot', 'Map[#^2 &, {1, 2, 3}]', '{1, 4, 9}')
        .done(),
    feature('Table', 'list')
        .supported()
        .pure()
        .notes('single iterator Table[i,{i,n}]')
        .eval('table.basic', 'Table[i, {i, 3}]', '{1, 2, 3}')
        .done(),
    feature('Sum', 'list')
        .supported()
        .pure()
        .notes('Sum over single iterator via Table fold')
        .eval('sum.basic', 'Sum[i, {i, 1, 10}]', '55')
        .done(),
    feature('Product', 'list')
        .supported()
        .pure()
        .notes('Product over single iterator via Table fold')
        .eval('product.basic', 'Product[i, {i, 1, 5}]', '120')
        .done(),
    feature('Length', 'list').supported().pure().eval('length.3', 'Length[{1, 2, 3}]', '3').done(),
    feature('First', 'list')
        .supported()
        .pure()
        .notes('First on non-empty List; empty → InvalidIndex at Athena')
        .eval('first.ab', 'First[{a, b}]', 'a')
        .done(),
    feature('Join', 'list').supported().pure().eval('join.basic', 'Join[{1}, {2}]', '{1, 2}').done(),
    feature('Flatten', 'list').supported().pure().eval('flatten.basic', 'Flatten[{{1, 2}, {3}}]', '{1, 2, 3}').done(),
    feature('Apply', 'list').supported().pure().eval('apply.plus', 'Apply[Plus, {1, 2, 3}]', '6').done(),
    feature('Rest', 'list').supported().pure().eval('rest.basic', 'Rest[{1, 2, 3}]', '{2, 3}').done(),
    feature('Most', 'list').supported().pure().eval('most.basic', 'Most[{1, 2, 3}]', '{1, 2}').done(),
    feature('Take', 'list').supported().pure().eval('take.2', 'Take[{1, 2, 3, 4}, 2]', '{1, 2}').done(),
    feature('Drop', 'list').supported().pure().eval('drop.2', 'Drop[{1, 2, 3, 4}, 2]', '{3, 4}').done(),
    feature('Reverse', 'list').supported().pure().eval('reverse.3', 'Reverse[{1, 2, 3}]', '{3, 2, 1}').done(),
    feature('Sort', 'list')
        .supported()
        .pure()
        .notes('Contract: exact-integer ascending only; mixed/non-integer → residual (not full MMA Sort)')
        .eval('sort.3', 'Sort[{3, 1, 2}]', '{1, 2, 3}')
        .done(),
    feature('MemberQ', 'list')
        .supported()
        .pure()
        .notes('Contract: structural equality membership only (not pattern `MemberQ`)')
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
        .supported()
        .pure()
        .notes('CollectMatches via Blank[Integer]; also covered under pattern.blank')
        .eval('cases.integer', 'Cases[{1, 2, 3}, _Integer]', '{1, 2, 3}')
        .eval('cases.mixed', 'Cases[{1, a, 2}, _Integer]', '{1, 2}')
        .done(),
    feature('Count', 'list')
        .supported()
        .pure()
        .notes('Contract: structural equality occurrence count (not pattern `Count`)')
        .eval('count.1', 'Count[{1, 1, 2}, 1]', '2')
        .done(),
    feature('Partition', 'list').supported().pure().eval('partition.2', 'Partition[{1, 2, 3, 4}, 2]', '{{1, 2}, {3, 4}}').done(),
    feature('Union', 'list')
        .supported()
        .pure()
        .notes('Contract: structural dedupe of list args; exact integers sorted ascending')
        .eval('union.basic', 'Union[{1, 2}, {2, 3}]', '{1, 2, 3}')
        .done(),
    feature('Intersection', 'list')
        .supported()
        .pure()
        .notes('Contract: structural intersection; exact integers sorted ascending')
        .eval('intersection.basic', 'Intersection[{1, 2}, {2, 3}]', '{2}')
        .done(),
    feature('FreeQ', 'list')
        .supported()
        .pure()
        .notes('Contract: top-level structural non-membership only (not deep/pattern `FreeQ`)')
        .eval('freeq.3', 'FreeQ[{1, 2}, 3]', 'True')
        .done(),
    feature('Position', 'list')
        .supported()
        .pure()
        .notes('Contract: top-level structural_eq positions as `{{i},…}` (not deep/pattern `Position`)')
        .eval('position.1', 'Position[{1, 2, 1}, 1]', '{{1}, {3}}')
        .done(),
    feature('Extract', 'list').supported().pure().eval('extract.2', 'Extract[{1, 2, 3}, 2]', '2').done(),
    feature('PadLeft', 'list')
        .supported()
        .pure()
        .notes('Contract: left-pad exact integer `0` to length `n`, or left-truncate (no pad value / level args)')
        .eval('padleft.4', 'PadLeft[{1, 2}, 4]', '{0, 0, 1, 2}')
        .done(),
    feature('Riffle', 'list')
        .supported()
        .pure()
        .notes('Contract: top-level zip of two lists, length = min (not MMA multi-arg / x-spacer `Riffle`)')
        .eval('riffle.ab', 'Riffle[{1, 2}, {a, b}]', '{1, a, 2, b}')
        .done(),
    feature('Accumulate', 'list').supported().pure().eval('accumulate.3', 'Accumulate[{1, 2, 3}]', '{1, 3, 6}').done(),
    feature('Differences', 'list').supported().pure().eval('differences.3', 'Differences[{1, 4, 9}]', '{3, 5}').done(),
    feature('Total', 'list').supported().pure().notes('Total lowers to Sum').eval('total.3', 'Total[{1, 2, 3}]', '6').done(),
    feature('Append', 'list').supported().pure().eval('append.3', 'Append[{1, 2}, 3]', '{1, 2, 3}').done(),
    feature('Prepend', 'list').supported().pure().eval('prepend.1', 'Prepend[{2, 3}, 1]', '{1, 2, 3}').done(),
    feature('DeleteDuplicates', 'list').supported().pure().eval('deletedup.112', 'DeleteDuplicates[{1, 1, 2}]', '{1, 2}').done(),
    feature('Array', 'list')
        .supported()
        .pure()
        .notes('Contract: `Array[f, n]` → `{f[1],…,f[n]}` for operator-value head and exact `n` (no dims/list specs)')
        .eval('array.f3', 'Array[f, 3]', '{f[1], f[2], f[3]}')
        .done(),
    feature('ConstantArray', 'list').supported().pure().eval('constarray.0', 'ConstantArray[0, 3]', '{0, 0, 0}').done(),
    feature('DeleteCases', 'list')
        .supported()
        .pure()
        .notes('CollectRejects via Blank[Integer]; inverse of Cases')
        .eval('deletecases.int', 'DeleteCases[{1, a, 2}, _Integer]', '{a}')
        .done(),
    feature('MapIndexed', 'list')
        .supported()
        .pure()
        .notes('Multi-slot `#2&` → `Function[{$slot1,$slot2},…]`; MapIndexed applies `f[elem,{i}]`')
        .eval('mapindexed.slot2', 'MapIndexed[#2 &, {a, b}]', '{{1}, {2}}')
        .done(),
    feature('MapThread', 'list')
        .supported()
        .pure()
        .notes('Zip columns of `{list1, list2, …}` then apply head / Function')
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
