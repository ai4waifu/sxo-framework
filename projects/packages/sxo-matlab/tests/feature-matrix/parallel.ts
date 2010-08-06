import { feature } from '@sxo/harness';

export const parallelFeatures = [
    feature('spmd', 'parallel')
        .unsupported('parse keeps Spmd Form via oak Statement::Spmd; parallel block runtime still open')
        .stateful()
        .gap('spmd.strip', 'spmd, 1, end', {
            expected: '...',
            notes: 'Form Spmd[1]; eval Reject (was silent → 1)',
        })
        .done(),
];
