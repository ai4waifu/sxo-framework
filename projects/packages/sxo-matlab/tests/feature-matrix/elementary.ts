import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('sin', 'elementary').supported().pure().eval('sin.0', 'sin(0)', '0').done(),
    feature('cos', 'elementary').supported().pure().eval('cos.0', 'cos(0)', '1').done(),
    feature('sqrt', 'elementary').supported().pure().eval('sqrt.4', 'sqrt(4)', '2').done(),
    feature('abs', 'elementary').supported().pure().eval('abs.neg', 'abs(-3)', '3').done(),
    feature('exp', 'elementary')
        .partial('pinned Athena after `fc860cf0` no longer exact-folds `Exp[0]`')
        .pure()
        .wrong('exp.0', 'exp(0)', {
            expected: '1',
            flags: ['upstream-athena'],
            notes: 'got unevaluated `exp(0)` on pin `53a7a5dd`',
        })
        .done(),
    feature('log', 'elementary')
        .partial('pinned Athena after `fc860cf0` no longer exact-folds `Log[1]`')
        .pure()
        .wrong('log.1', 'log(1)', {
            expected: '0',
            flags: ['upstream-athena'],
            notes: 'got unevaluated `log(1)` on pin `53a7a5dd`',
        })
        .done(),
    feature('listable_sin', 'elementary')
        .unsupported('sin on vector left as sin([...])')
        .pure()
        .gap('sin.listable', 'sin([0, pi/2])', { expected: '[0, 1]' })
        .done(),
];
