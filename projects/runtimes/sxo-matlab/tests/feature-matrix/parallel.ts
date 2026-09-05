import { feature } from '@sxo/harness';

export const parallelFeatures = [
    feature('spmd', 'parallel')
        .unsupported('SILENT WRONG: spmd, 1, end → 1')
        .stateful()
        .gap('spmd.strip', 'spmd, 1, end', { expected: '...', notes: 'currently returns 1' })
        .done(),
];
