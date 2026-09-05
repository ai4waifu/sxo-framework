import { feature } from '@sxo/harness';

export const statsFeatures = [feature('Mean', 'stats').unsupported().pure().gap('mean.3', 'Mean[{1, 2, 3}]', { expected: '2' }).done()];
