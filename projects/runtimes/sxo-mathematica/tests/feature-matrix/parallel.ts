import { feature } from '@sxo/harness';

export const parallelFeatures = [
    feature('ParallelEvaluate', 'parallel')
        .unsupported('SILENT WRONG eval-early: ParallelEvaluate[1+1] → ParallelEvaluate[2]')
        .effectful()
        .gap('paralleleval.plus', 'ParallelEvaluate[1 + 1]', { expected: '2', notes: 'currently ParallelEvaluate[2]' })
        .done(),
];
