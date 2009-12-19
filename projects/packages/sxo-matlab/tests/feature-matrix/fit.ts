import { feature } from '@sxo/harness';

export const fitFeatures = [
    feature('polyfit', 'fit').unsupported().pure().gap('polyfit.quad', 'polyfit([1, 2, 3], [1, 4, 9], 2)', { expected: '[1, 0, 0]' }).done(),
    feature('interp1', 'fit').unsupported().pure().gap('interp1.mid', 'interp1([0, 1], [0, 1], 0.5)', { expected: '0.5' }).done(),
];
