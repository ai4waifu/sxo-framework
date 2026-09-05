import { feature } from '@sxo/harness';

export const bitwiseFeatures = [
    feature('bitand', 'bitwise').unsupported().pure().gap('bitand.63', 'bitand(6, 3)', { expected: '2' }).done(),
    feature('bitcmp', 'bitwise').unsupported().pure().gap('bitcmp.u8', "bitcmp(1, 'uint8')", { expected: '254' }).done(),
];
