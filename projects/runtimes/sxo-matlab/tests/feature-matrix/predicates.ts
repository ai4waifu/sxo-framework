import { feature } from '@sxo/harness';

export const predicatesFeatures = [
    feature('isnan', 'predicates')
        .unsupported()
        .pure()
        .gap('isnan.nan', 'isnan(NaN)', { expected: '1' })
        .gap('isnan.0', 'isnan(0)', { expected: '0' })
        .done(),
];
