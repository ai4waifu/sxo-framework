import { feature } from '@sxo/harness';

export const tableFeatures = [
    feature('outerjoin', 'table')
        .unsupported('parse keeps cell VariableNames via oak CellArray; table/join runtime still open')
        .pure()
        .gap('outerjoin.k', "outerjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", {
            expected: '...',
            notes: 'Form keeps Cell({k}); runtime pending',
        })
        .done(),
    feature('leftjoin', 'table')
        .unsupported('same cell VariableNames Form fidelity as outerjoin; join runtime still open')
        .pure()
        .gap('leftjoin.k', "leftjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", {
            expected: '...',
            notes: 'Form keeps Cell({k}); runtime pending',
        })
        .done(),
];
