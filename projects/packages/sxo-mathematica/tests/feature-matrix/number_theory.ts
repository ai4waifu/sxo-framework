import { feature } from '@sxo/harness';

export const numberTheoryFeatures = [
    feature('GCD', 'number_theory').unsupported().pure().gap('gcd.basic', 'GCD[12, 18]', { expected: '6' }).done(),
    feature('Mod', 'number_theory').unsupported().pure().gap('mod.basic', 'Mod[10, 3]', { expected: '1' }).done(),
    feature('PrimeQ', 'number_theory').unsupported().pure().gap('primeq.7', 'PrimeQ[7]', { expected: 'True' }).done(),
    feature('Binomial', 'number_theory').unsupported().pure().gap('binomial.52', 'Binomial[5, 2]', { expected: '10' }).done(),
    feature('LCM', 'number_theory').unsupported().pure().gap('lcm.46', 'LCM[4, 6]', { expected: '12' }).done(),
    feature('IntegerDigits', 'number_theory')
        .unsupported()
        .pure()
        .gap('integerdigits.123', 'IntegerDigits[123]', { expected: '{1, 2, 3}' })
        .done(),
    feature('FromDigits', 'number_theory').unsupported().pure().gap('fromdigits.123', 'FromDigits[{1, 2, 3}]', { expected: '123' }).done(),
    feature('Prime', 'number_theory').unsupported().pure().gap('prime.10', 'Prime[10]', { expected: '29' }).done(),
    feature('FactorInteger', 'number_theory')
        .unsupported()
        .pure()
        .gap('factorinteger.12', 'FactorInteger[12]', { expected: '{{2, 2}, {3, 1}}' })
        .done(),
    feature('PowerMod', 'number_theory').unsupported().pure().gap('powermod.basic', 'PowerMod[2, 10, 7]', { expected: '2' }).done(),
    feature('ChineseRemainder', 'number_theory')
        .unsupported()
        .pure()
        .gap('crt.basic', 'ChineseRemainder[{1, 2}, {3, 5}]', { expected: '7' })
        .done(),
    feature('Fibonacci', 'number_theory').unsupported().pure().gap('fibonacci.10', 'Fibonacci[10]', { expected: '55' }).done(),
];
