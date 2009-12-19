import { feature } from '@sxo/harness';

export const arithmeticFeatures = [
    feature('plus', 'arithmetic').supported().pure().eval('plus.basic', '2 + 3', '5').done(),
    feature('mtimes', 'arithmetic')
        .partial('scalar * and numeric nested-list matmul work; symbolic matrix * stays Times')
        .pure()
        .eval('mtimes.scalar', '2 * 3', '6')
        .gap('mtimes.2x2', '[1, 2; 3, 4]*[5, 6; 7, 8]', { expected: '[19, 22; 43, 50]', notes: 'currently [1, 2; 3, 4]*[5, 6; 7, 8]' })
        .done(),
    feature('times', 'arithmetic')
        .supported()
        .pure()
        .notes('.* → DotTimes elementwise')
        .eval('times.scalar', '2 .* [1, 2]', '[2, 4]')
        .eval('times.vec', '[1, 2].*[3, 4]', '[3, 8]')
        .done(),
    feature('power', 'arithmetic')
        .partial('scalar ^ and .^ OK; (x+1)^2 still expands')
        .pure()
        .eval('power.basic', '2^3', '8')
        .eval('power.elementwise', '[1, 2].^[2, 3]', '[1, 8]')
        .gap('power.binomsq', '(x + 1)^2', { expected: '(x + 1)^2', notes: 'currently expands' })
        .eval('power.vec_pow0', '[1, 2, 3].^0', '[1, 1, 1]')
        .done(),
    feature('mrdivide', 'arithmetic').supported().pure().eval('mrdivide.basic', '6 / 2', '3').done(),
    feature('rdivide', 'arithmetic')
        .supported()
        .pure()
        .notes('./ → DotDivide')
        .eval('rdivide.scalar', '1./2', '0.5')
        .eval('rdivide.vec', '[6, 8]./[2, 4]', '[3, 2]')
        .done(),
];
