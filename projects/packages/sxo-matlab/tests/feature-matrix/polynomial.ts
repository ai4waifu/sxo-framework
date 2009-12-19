import { feature } from '@sxo/harness';

export const polynomialFeatures = [
    feature('polyval', 'polynomial').unsupported().pure().gap('polyval.quad', 'polyval([1, 0, -1], 2)', { expected: '3' }).done(),
    feature('polyder', 'polynomial').unsupported().pure().gap('polyder.quad', 'polyder([1, 2, 1])', { expected: '[2, 2]' }).done(),
];
