import { feature } from '@sxo/harness';

/** Bootstrap arithmetic surface — planned until oak + GpForm + Athena lowering land. */
export const arithmeticFeatures = [
    feature('Plus', 'arithmetic')
        .planned('PARI/GP Plus not wired; keep matrix entry for release tracking')
        .pure()
        .gap('plus.1_1', '1+1', { expected: '2', notes: 'needs GpForm parse + Athena Plus' })
        .done(),
    feature('Factor', 'arithmetic')
        .planned('integer factorization surface tracked for PARI/GP dialect')
        .pure()
        .gap('factor.6', 'factor(6)', { expected: '[2, 3]', notes: 'representative PARI-style call' })
        .done(),
];
