import { feature } from '@sxo/harness';

export const calculusFeatures = [
    feature('diff', 'calculus')
        .partial('first derivative works; higher-order diff(f,x,2) unevaluated')
        .pure()
        .eval('diff.poly', 'diff(x^3, x)', '3*x^2')
        .eval('diff.sin', 'diff(sin(x), x)', 'cos(x)')
        .gap('diff.order2', 'diff(x^2, x, 2)', { expected: '2' })
        .done(),
    feature('int', 'calculus')
        .supported()
        .pure()
        .eval('int.poly', 'int(x^2, x)', '1/3*x^3')
        .eval('int.sin', 'int(sin(x), x)', '-cos(x)')
        .done(),
    feature('integral', 'calculus')
        .unsupported('@ stripped to integral(x, sin(x), 0, pi)')
        .pure()
        .gap('integral.sin', 'integral(@(x)sin(x), 0, pi)', { expected: '2' })
        .done(),
    feature('quadgk', 'calculus')
        .unsupported('@ stripped to quadgk(x, sin(x), 0, pi)')
        .pure()
        .gap('quadgk.sin', 'quadgk(@(x)sin(x), 0, pi)', { expected: '2' })
        .done(),
];
