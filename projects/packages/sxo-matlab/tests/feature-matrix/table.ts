import { feature } from '@sxo/harness';

export const tableFeatures = [
    feature('outerjoin', 'table')
        .unsupported('cell brace in call args refused until oak CellArray (was silent VariableNames strip)')
        .pure()
        .gap('outerjoin.k', "outerjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", {
            expected: '...',
            notes: 'parse error: unsupported cell brace in call/index',
        })
        .done(),
    feature('leftjoin', 'table')
        .unsupported('same cell-brace refuse as outerjoin')
        .pure()
        .gap('leftjoin.k', "leftjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", {
            expected: '...',
            notes: 'parse error: unsupported cell brace in call/index',
        })
        .done(),
];
