import { feature } from '@sxo/harness';

export const tableFeatures = [
    feature('outerjoin', 'table')
        .unsupported("SILENT WRONG: cell {'k'} in VariableNames stripped to 'k'")
        .pure()
        .gap('outerjoin.k', "outerjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", {
            expected: '...',
            notes: "currently VariableNames 'k' without cell",
        })
        .done(),
    feature('leftjoin', 'table')
        .unsupported('same cell VariableNames strip as outerjoin')
        .pure()
        .gap('leftjoin.k', "leftjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", { expected: '...' })
        .done(),
];
