import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('sin', 'elementary').partial('scalar `sin` on tested exact arguments only').pure().eval('sin.0', 'sin(0)', '0').done(),
    feature('cos', 'elementary').partial('scalar `cos` on tested exact arguments only').pure().eval('cos.0', 'cos(0)', '1').done(),
    feature('sqrt', 'elementary').partial('scalar `sqrt` on tested exact squares only').pure().eval('sqrt.4', 'sqrt(4)', '2').done(),
    feature('abs', 'elementary').partial('scalar `abs` on tested exact integers only').pure().eval('abs.neg', 'abs(-3)', '3').done(),
    feature('exp', 'elementary').partial('scalar `exp` on tested exact arguments only').pure().eval('exp.0', 'exp(0)', '1').done(),
    feature('log', 'elementary').partial('scalar `log` on tested exact arguments only').pure().eval('log.1', 'log(1)', '0').done(),
    feature('listable_sin', 'elementary')
        .unsupported('sin on vector left as sin([...])')
        .pure()
        .gap('sin.listable', 'sin([0, pi/2])', { expected: '[0, 1]' })
        .done(),
];
