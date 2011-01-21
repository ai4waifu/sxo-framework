import { feature } from '@sxo/harness';

export const calculusFeatures = [
    feature('diff', 'calculus')
        .partial('symbolic `diff` on tested polynomial and trig forms only')
        .pure()
        .eval('diff.poly', 'diff(x^3, x)', '3*x^2')
        .eval('diff.sin', 'diff(sin(x), x)', 'cos(x)')
        .eval('diff.order2', 'diff(x^2, x, 2)', '2')
        .done(),
    feature('int', 'calculus')
        .partial('indefinite `int` on tested polynomial and trig forms only')
        .pure()
        .eval('int.poly', 'int(x^2, x)', 'x^3*3^(-1)', {
            notes: 'canonical Power form; mathematically equivalent to `1/3*x^3`',
        })
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
