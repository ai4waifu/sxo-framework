import { feature } from '@sxo/harness';

export const parseFeatures = [
    feature('JuxtapositionTimes', 'parse')
        .partial('implicit `Times` via juxtaposition on tested forms only')
        .pure()
        .eval('juxt.d', 'D[x y, x]', 'y')
        .done(),
    feature('Prefix', 'parse').partial('`f@x` → `f[x]` on tested symbols only').pure().eval('prefix.fx', 'f@x', 'f[x]').done(),
    feature('Postfix', 'parse').partial('`x//f` → `f[x]` on tested symbols only').pure().eval('postfix.xf', 'x//f', 'f[x]').done(),
    feature('ScientificLiteral', 'parse')
        .unsupported('oak error on 1*^3 and 2.5*^-2')
        .pure()
        .gap('sci.1e3', '1*^3', { expected: '1000' })
        .gap('sci.2p5em2', '2.5*^-2', { expected: '0.025' })
        .done(),
    feature('BaseLiteral', 'parse')
        .unsupported('oak error on 16^^FF and 2^^1010')
        .pure()
        .gap('base.hex', '16^^FF', { expected: '255' })
        .gap('base.bin', '2^^1010', { expected: '10' })
        .done(),
];
