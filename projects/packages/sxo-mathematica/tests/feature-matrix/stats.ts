import { feature } from '@sxo/harness';

export const statsFeatures = [
    feature('Mean', 'stats').partial('exact `Mean[{1, 2, 3}]` on flat numeric lists').pure().eval('mean.3', 'Mean[{1, 2, 3}]', '2').done(),
];
