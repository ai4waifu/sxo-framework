import { feature } from '@sxo/harness';

export const parallelFeatures = [
    feature('ParallelEvaluate', 'parallel')
        .unsupported('HoldAll Form kept; no parallel scheduler runtime')
        .effectful()
        .gap('paralleleval.plus', 'ParallelEvaluate[1 + 1]', {
            expected: '2',
            notes: 'must stay ParallelEvaluate[1 + 1], not ParallelEvaluate[2]; scheduler still unsupported',
        })
        .done(),
];
