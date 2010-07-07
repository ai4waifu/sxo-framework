import { feature } from '@sxo/harness';

/** Classic GP number-theory builtins — planned until Athena number-theory domains land. */
export const numberTheoryFeatures = [
    feature('factor', 'number_theory')
        .planned('integer factorization surface tracked for PARI/GP dialect')
        .pure()
        .gap('factor.6', 'factor(6)', { expected: '[2, 3]', notes: 'representative PARI-style call' })
        .done(),
    feature('gcd', 'number_theory')
        .planned('gcd via Athena exact integer domain')
        .pure()
        .gap('gcd.basic', 'gcd(12, 18)', { expected: '6' })
        .done(),
    feature('isprime', 'number_theory')
        .planned('primality predicate')
        .pure()
        .gap('isprime.17', 'isprime(17)', { expected: '1' })
        .gap('isprime.15', 'isprime(15)', { expected: '0' })
        .done(),
    feature('nextprime', 'number_theory').planned('nextprime surface').pure().gap('nextprime.10', 'nextprime(10)', { expected: '11' }).done(),
    feature('eulerphi', 'number_theory').planned("Euler's totient").pure().gap('eulerphi.9', 'eulerphi(9)', { expected: '6' }).done(),
];
