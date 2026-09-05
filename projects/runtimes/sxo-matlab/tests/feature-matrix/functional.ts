import { feature } from '@sxo/harness';

export const functionalFeatures = [
    feature('compose', 'functional').unsupported().pure().gap('compose.sincos', 'compose(sin, cos, 0)', { expected: '0' }).done(),
];
