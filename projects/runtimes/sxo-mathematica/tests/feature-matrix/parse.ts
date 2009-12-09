import { feature } from '@sxo/harness';

export const parseFeatures = [
    feature('JuxtapositionTimes', 'parse')
        .unsupported('implicit Times often becomes arg-splitting in D/Integrate/Collect (SILENT WRONG)')
        .pure()
        .gap('juxt.d', 'D[x y, x]', { expected: 'y', notes: 'currently D[x, y, x]' })
        .gap('juxt.collect', 'Collect[x^2 + 2*x*y + y^2, x]', { expected: 'x^2 + 2*x*y + y^2' })
        .done(),
    feature('Prefix', 'parse').supported().pure().notes('f@x → f[x]').eval('prefix.fx', 'f@x', 'f[x]').done(),
    feature('Postfix', 'parse').supported().pure().notes('x//f → f[x]').eval('postfix.xf', 'x//f', 'f[x]').done(),
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
