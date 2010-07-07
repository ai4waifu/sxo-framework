import { feature } from '@sxo/harness';

/** Exact arithmetic surface — planned until oak + GpForm + Athena lowering land. */
export const arithmeticFeatures = [
    feature('Plus', 'arithmetic')
        .planned('PARI/GP Plus not wired; keep matrix entry for release tracking')
        .pure()
        .gap('plus.1_1', '1+1', { expected: '2', notes: 'needs GpForm parse + Athena Plus' })
        .gap('plus.big', '10^20 + 1', { expected: '100000000000000000001', notes: 'exact BigInteger path' })
        .done(),
    feature('Multiply', 'arithmetic')
        .planned('exact multiply tracked for PARI/GP dialect')
        .pure()
        .gap('mul.basic', '6*7', { expected: '42' })
        .done(),
    feature('Divide', 'arithmetic')
        .planned('exact / rational divide')
        .pure()
        .gap('div.rat', '1/3', { expected: '1/3', notes: 'prefer exact rational over float' })
        .done(),
    feature('Power', 'arithmetic').planned('integer power').pure().gap('pow.int', '2^10', { expected: '1024' }).done(),
];
