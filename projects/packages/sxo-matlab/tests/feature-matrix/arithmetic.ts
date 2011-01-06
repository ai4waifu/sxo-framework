import { feature } from '@sxo/harness';

export const arithmeticFeatures = [
    feature('plus', 'arithmetic').supported().pure().eval('plus.basic', '2 + 3', '5').done(),
    feature('parens', 'arithmetic')
        .supported()
        .pure()
        .notes('R-2.11: string and handle paths must preserve grouping (no display-text fork)')
        .eval('parens.mul', '(1+2)*3', '9')
        .eval('parens.div', '1/(2+3)', '1/5')
        .eval('parens.sub', '1-(2-3)', '2')
        .eval('parens.dottimes', '[1,2].*(3+4)', '[7, 14]')
        .done(),
    feature('mtimes', 'arithmetic')
        .supported()
        .pure()
        .notes('scalar * and typed matrix MatMul via Living 16 MatrixOperand')
        .eval('mtimes.scalar', '2 * 3', '6')
        .eval('mtimes.2x2', '[1, 2; 3, 4]*[5, 6; 7, 8]', '[19, 22; 43, 50]')
        .done(),
    feature('times', 'arithmetic')
        .supported()
        .pure()
        .notes('.* → Hadamard / DotTimes elementwise')
        .eval('times.scalar', '2 .* [1, 2]', '[2, 4]')
        .eval('times.vec', '[1, 2].*[3, 4]', '[3, 8]')
        .eval('times.mat', '[1, 2; 3, 4].*[5, 6; 7, 8]', '[5, 12; 21, 32]')
        .done(),
    feature('power', 'arithmetic')
        .supported()
        .pure()
        .notes('scalar ^ and .^ → ElementwisePower on typed matrices')
        .eval('power.basic', '2^3', '8')
        .eval('power.elementwise', '[1, 2].^[2, 3]', '[1, 8]')
        .eval('power.mat', '[2, 3; 4, 5].^[2, 2; 2, 2]', '[4, 9; 16, 25]')
        .eval('power.binomsq', '(x + 1)^2', '(1 + x)^2')
        .eval('power.vec_pow0', '[1, 2, 3].^0', '[1, 1, 1]')
        .done(),
    feature('mrdivide', 'arithmetic')
        .partial('scalar `/` stays Divide. Typed matrices → RightSolve with disposition residuals (see solve.mrdivide)')
        .pure()
        .eval('mrdivide.basic', '6 / 2', '3')
        .eval('mrdivide.row', '[1, 2] / [1, 2; 3, 4]', '[[1, 0]]')
        .eval('mrdivide.2x2', '[1, 2; 3, 4] / [1, 2; 3, 4]', '[1, 0; 0, 1]')
        .done(),
    feature('rdivide', 'arithmetic')
        .supported()
        .pure()
        .notes('./ → ElementwiseDivide on typed matrices')
        .eval('rdivide.scalar', '1./2', '0.5')
        .eval('rdivide.vec', '[6, 8]./[2, 4]', '[3, 2]')
        .eval('rdivide.mat', '[6, 8; 10, 12]./[2, 4; 5, 6]', '[3, 2; 2, 2]')
        .done(),
];
