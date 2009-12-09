import { feature } from '@sxo/harness';

export const typesFeatures = [
    feature('cell', 'types')
        .unsupported('SILENT WRONG: {1,2} evaluates to 2 (brace not cell)')
        .pure()
        .gap('cell.literal', '{1, 2}', { expected: '{1, 2}', notes: 'currently returns 2' })
        .gap('cell.ctor', 'cell(2, 1)', { expected: '{[]; []}' })
        .done(),
    feature('struct', 'types').unsupported().pure().gap('struct.basic', "struct('a', 1)", { expected: "struct('a',1)" }).done(),
    feature('class', 'types').unsupported().pure().gap('class.double', 'class(1)', { expected: "'double'" }).done(),
    feature('isa', 'types').unsupported().pure().gap('isa.double', "isa(1, 'double')", { expected: '1' }).done(),
    feature('isnumeric', 'types').unsupported().pure().gap('isnumeric.1', 'isnumeric(1)', { expected: '1' }).done(),
    feature('datetime', 'types').unsupported().pure().gap('datetime.ymd', 'datetime(2020, 1, 1)', { expected: '...' }).done(),
    feature('containers_Map', 'types')
        .unsupported('SILENT WRONG: containers.Map → Map (package path stripped)')
        .pure()
        .gap('containers.map', 'containers.Map', { expected: 'containers.Map', notes: 'currently returns Map' })
        .done(),
    feature('iscell', 'types')
        .unsupported('SILENT WRONG: iscell({1}) → iscell(1) (brace cell stripped first)')
        .pure()
        .gap('iscell.brace', 'iscell({1})', { expected: '1', notes: 'currently iscell(1)' })
        .done(),
    feature('missing', 'types')
        .partial('atom retained; ismissing/rmmissing unevaluated')
        .pure()
        .eval('missing.atom', 'missing', 'missing')
        .done(),
    feature('ismissing', 'types').unsupported().pure().gap('ismissing.missing', 'ismissing(missing)', { expected: '1' }).done(),
    feature('logical', 'types').unsupported().pure().gap('logical.vec', 'logical([1, 0, 2])', { expected: '[1, 0, 1]' }).done(),
    feature('isempty', 'types')
        .unsupported()
        .pure()
        .gap('isempty.empty', 'isempty([])', { expected: '1' })
        .gap('isempty.zero', 'isempty(0)', { expected: '0' })
        .done(),
    feature('isscalar', 'types').unsupported().pure().gap('isscalar.1', 'isscalar(1)', { expected: '1' }).done(),
    feature('isvector', 'types').unsupported().pure().gap('isvector.row', 'isvector([1, 2])', { expected: '1' }).done(),
    feature('isrow', 'types').unsupported().pure().gap('isrow.12', 'isrow([1, 2])', { expected: '1' }).done(),
    feature('iscolumn', 'types').unsupported().pure().gap('iscolumn.12', 'iscolumn([1; 2])', { expected: '1' }).done(),
    feature('fillmissing', 'types')
        .unsupported()
        .pure()
        .gap('fillmissing.const', "fillmissing([1, NaN], 'constant', 0)", { expected: '[1, 0]' })
        .done(),
    feature('isstring', 'types')
        .unsupported()
        .pure()
        .gap('isstring.dq', 'isstring("a")', { expected: '1' })
        .gap('isstring.sq', "isstring('a')", { expected: '0' })
        .done(),
    feature('ischar', 'types')
        .unsupported()
        .pure()
        .gap('ischar.sq', "ischar('a')", { expected: '1' })
        .gap('ischar.dq', 'ischar("a")', { expected: '0' })
        .done(),
];
