import { feature } from '@sxo/harness';

export const statsFeatures = [
    feature('mean', 'stats').unsupported().pure().gap('mean.vec', 'mean([1, 2, 3])', { expected: '2' }).done(),
    feature('std', 'stats').unsupported().pure().gap('std.vec', 'std([1, 2, 3])', { expected: '1' }).done(),
    feature('normcdf', 'stats').unsupported().pure().gap('normcdf.0', 'normcdf(0)', { expected: '0.5' }).done(),
];
