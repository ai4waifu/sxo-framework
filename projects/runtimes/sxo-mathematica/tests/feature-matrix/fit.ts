import { feature } from '@sxo/harness';

export const fitFeatures = [
    feature('Fit', 'fit').unsupported().pure().gap('fit.linear', 'Fit[{1, 2}, {1, x}, x]', { expected: '...' }).done(),
    feature('FindFit', 'fit').unsupported().pure().gap('findfit.ax', 'FindFit[{1, 4}, {a*x}, {a}, x]', { expected: '{a -> 2.}' }).done(),
];
