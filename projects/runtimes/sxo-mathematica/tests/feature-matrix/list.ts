import { feature } from '@sxo/harness';

export const listFeatures = [
    feature('List', 'list')
        .supported()
        .pure()
        .eval('list.literal', '{1, 2, 3}', '{1, 2, 3}')
        .roundtrip('list.roundtrip', '{1, 2}', '{1, 2}')
        .done(),
    feature('Part', 'list')
        .supported()
        .pure()
        .notes('1-based Part; [[0]] returns head List')
        .eval('part.infix', '{1, 2, 3}[[2]]', '2')
        .eval('part.head', 'Part[{1, 2, 3}, 1]', '1')
        .eval('part.zero', '{1, 2, 3}[[0]]', 'List')
        .done(),
    feature('Range', 'list').supported().pure().notes('Range[n] integer sequence').eval('range.3', 'Range[3]', '{1, 2, 3}').done(),
    feature('Map', 'list')
        .partial('Slot Map OK; Map[Sin,{0,1}] currently numericizes Sin[1]')
        .pure()
        .gap('map.sin', 'Map[Sin, {0, 1}]', { expected: '{0, Sin[1]}', notes: 'currently {0, 0.8414709848078965}' })
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
    feature('Flatten', 'list').unsupported().pure().gap('flatten.basic', 'Flatten[{{1, 2}, {3}}]', { expected: '{1, 2, 3}' }).done(),
    feature('Apply', 'list').supported().pure().eval('apply.plus', 'Apply[Plus, {1, 2, 3}]', '6').done(),
    feature('Rest', 'list').unsupported().pure().gap('rest.basic', 'Rest[{1, 2, 3}]', { expected: '{2, 3}' }).done(),
    feature('Most', 'list').unsupported().pure().gap('most.basic', 'Most[{1, 2, 3}]', { expected: '{1, 2}' }).done(),
    feature('Take', 'list').unsupported().pure().gap('take.2', 'Take[{1, 2, 3, 4}, 2]', { expected: '{1, 2}' }).done(),
    feature('Drop', 'list').unsupported().pure().gap('drop.2', 'Drop[{1, 2, 3, 4}, 2]', { expected: '{3, 4}' }).done(),
    feature('Reverse', 'list').unsupported().pure().gap('reverse.3', 'Reverse[{1, 2, 3}]', { expected: '{3, 2, 1}' }).done(),
    feature('Sort', 'list').unsupported().pure().gap('sort.3', 'Sort[{3, 1, 2}]', { expected: '{1, 2, 3}' }).done(),
    feature('MemberQ', 'list').unsupported().pure().gap('memberq.2', 'MemberQ[{1, 2, 3}, 2]', { expected: 'True' }).done(),
    feature('Select', 'list')
        .unsupported('SILENT WRONG: Select[{1,2,3,4},EvenQ] → EvenQ')
        .pure()
        .gap('select.evenq', 'Select[{1, 2, 3, 4}, EvenQ]', { expected: '{2, 4}', notes: 'currently returns EvenQ' })
        .done(),
    feature('Cases', 'list')
        .unsupported('SILENT WRONG: Cases[{1,2,3},_Integer] → Integer (Blank stripped)')
        .pure()
        .gap('cases.integer', 'Cases[{1, 2, 3}, _Integer]', { expected: '{1, 2, 3}', notes: 'currently returns Integer' })
        .done(),
    feature('Count', 'list').unsupported().pure().gap('count.1', 'Count[{1, 1, 2}, 1]', { expected: '2' }).done(),
    feature('Partition', 'list').unsupported().pure().gap('partition.2', 'Partition[{1, 2, 3, 4}, 2]', { expected: '{{1, 2}, {3, 4}}' }).done(),
    feature('Union', 'list').unsupported().pure().gap('union.basic', 'Union[{1, 2}, {2, 3}]', { expected: '{1, 2, 3}' }).done(),
    feature('Intersection', 'list').unsupported().pure().gap('intersection.basic', 'Intersection[{1, 2}, {2, 3}]', { expected: '{2}' }).done(),
    feature('FreeQ', 'list').unsupported().pure().gap('freeq.3', 'FreeQ[{1, 2}, 3]', { expected: 'True' }).done(),
    feature('Position', 'list').unsupported().pure().gap('position.1', 'Position[{1, 2, 1}, 1]', { expected: '{{1}, {3}}' }).done(),
    feature('Extract', 'list').unsupported().pure().gap('extract.2', 'Extract[{1, 2, 3}, 2]', { expected: '2' }).done(),
    feature('PadLeft', 'list').unsupported().pure().gap('padleft.4', 'PadLeft[{1, 2}, 4]', { expected: '{0, 0, 1, 2}' }).done(),
    feature('Riffle', 'list').unsupported().pure().gap('riffle.ab', 'Riffle[{1, 2}, {a, b}]', { expected: '{1, a, 2, b}' }).done(),
    feature('Accumulate', 'list').unsupported().pure().gap('accumulate.3', 'Accumulate[{1, 2, 3}]', { expected: '{1, 3, 6}' }).done(),
    feature('Differences', 'list').unsupported().pure().gap('differences.3', 'Differences[{1, 4, 9}]', { expected: '{3, 5}' }).done(),
    feature('Total', 'list').unsupported().pure().gap('total.3', 'Total[{1, 2, 3}]', { expected: '6' }).done(),
    feature('Append', 'list').unsupported().pure().gap('append.3', 'Append[{1, 2}, 3]', { expected: '{1, 2, 3}' }).done(),
    feature('Prepend', 'list').unsupported().pure().gap('prepend.1', 'Prepend[{2, 3}, 1]', { expected: '{1, 2, 3}' }).done(),
    feature('DeleteDuplicates', 'list').unsupported().pure().gap('deletedup.112', 'DeleteDuplicates[{1, 1, 2}]', { expected: '{1, 2}' }).done(),
    feature('Array', 'list').unsupported().pure().gap('array.f3', 'Array[f, 3]', { expected: '{f[1], f[2], f[3]}' }).done(),
    feature('ConstantArray', 'list').unsupported().pure().gap('constarray.0', 'ConstantArray[0, 3]', { expected: '{0, 0, 0}' }).done(),
    feature('DeleteCases', 'list')
        .unsupported('Blank stripped: DeleteCases[{1,a,2},_Integer] → DeleteCases[..., Integer]')
        .pure()
        .gap('deletecases.int', 'DeleteCases[{1, a, 2}, _Integer]', { expected: '{a}' })
        .done(),
    feature('MapIndexed', 'list')
        .unsupported('SILENT WRONG parse of #2& → MapIndexed[#, 2 &, …]')
        .pure()
        .gap('mapindexed.slot2', 'MapIndexed[#2 &, {a, b}]', { expected: '{{1}, {2}}' })
        .done(),
    feature('MapThread', 'list')
        .unsupported()
        .pure()
        .gap('mapthread.f', 'MapThread[f, {{1, 2}, {3, 4}}]', { expected: '{f[1, 3], f[2, 4]}' })
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
