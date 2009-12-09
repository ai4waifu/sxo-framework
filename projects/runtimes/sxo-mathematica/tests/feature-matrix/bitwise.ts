import { feature } from '@sxo/harness';

export const bitwiseFeatures = [
    feature('BitAnd', 'bitwise').unsupported().pure().gap('bitand.63', 'BitAnd[6, 3]', { expected: '2' }).done(),
    feature('BitOr', 'bitwise').unsupported().pure().gap('bitor.12', 'BitOr[1, 2]', { expected: '3' }).done(),
];
