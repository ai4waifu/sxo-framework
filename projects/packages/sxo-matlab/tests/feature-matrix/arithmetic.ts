import { feature } from '@sxo/harness';

export const arithmeticFeatures = [
    feature('plus', 'arithmetic').partial('scalar exact `+` on tested integers only').pure().eval('plus.basic', '2 + 3', '5').done(),
    feature('parens', 'arithmetic')
        .partial('grouping and precedence on tested scalar and vector forms')
        .pure()
        .eval('parens.mul', '(1+2)*3', '9')
        .eval('parens.div', '1/(2+3)', '1/5')
        .eval('parens.sub', '1-(2-3)', '2')
        .eval('parens.dottimes', '[1,2].*(3+4)', '[7, 14]')
        .done(),
    feature('mtimes', 'arithmetic')
        .partial('scalar multiply and typed numeric matrix `MatMul` only')
        .pure()
        .eval('mtimes.scalar', '2 * 3', '6')
        .eval('mtimes.2x2', '[1, 2; 3, 4]*[5, 6; 7, 8]', '[19, 22; 43, 50]')
        .eval('mtimes.complex', '[1+i, 0; 0, 1-i]*[1, i; -i, 1]', '[1 + i, -1 + i; -1 - i, 1 - i]')
        .done(),
    feature('times', 'arithmetic')
        .partial('scalar `.*` and typed numeric Hadamard only')
        .pure()
        .eval('times.scalar', '2 .* [1, 2]', '[2, 4]')
        .eval('times.vec', '[1, 2].*[3, 4]', '[3, 8]')
        .eval('times.mat', '[1, 2; 3, 4].*[5, 6; 7, 8]', '[5, 12; 21, 32]')
        .eval('times.complex', '[1+i, 2; 3, 4].*[1, i; 0, 1]', '[1 + i, 2*i; 0, 4]')
        .done(),
    feature('power', 'arithmetic')
        .partial('scalar `^` and typed numeric elementwise `.^` only')
        .pure()
        .eval('power.basic', '2^3', '8')
        .eval('power.elementwise', '[1, 2].^[2, 3]', '[1, 8]')
        .eval('power.mat', '[2, 3; 4, 5].^[2, 2; 2, 2]', '[4, 9; 16, 25]')
        .eval('power.binomsq', '(x + 1)^2', '(1 + x)^2')
        .eval('power.vec_pow0', '[1, 2, 3].^0', '[1, 1, 1]')
        .eval('power.complex', '[1+i].^2', '2*i')
        .done(),
    feature('mrdivide', 'arithmetic')
        .partial('scalar `/` stays Divide. Typed matrices → RightSolve with disposition residuals (see solve.mrdivide)')
        .pure()
        .eval('mrdivide.basic', '6 / 2', '3')
        .eval('mrdivide.row', '[1, 2] / [1, 2; 3, 4]', '[[1, 0]]')
        .eval('mrdivide.2x2', '[1, 2; 3, 4] / [1, 2; 3, 4]', '[1, 0; 0, 1]')
        .done(),
    feature('rdivide', 'arithmetic')
        .partial('scalar `./` and typed numeric elementwise divide only')
        .pure()
        .eval('rdivide.scalar', '1./2', '0.5')
        .eval('rdivide.vec', '[6, 8]./[2, 4]', '[3, 2]')
        .eval('rdivide.mat', '[6, 8; 10, 12]./[2, 4; 5, 6]', '[3, 2; 2, 2]')
        .done(),
];
