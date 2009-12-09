import { feature } from '@sxo/harness';

export const arithmeticFeatures = [
    feature('Plus', 'arithmetic').supported().pure().eval('plus.basic', '1 + 2 * 3', '7').eval('plus.nary', 'Plus[1, 2, 3]', '6').done(),
    feature('Times', 'arithmetic').supported().pure().eval('times.nary', 'Times[2, 3, 4]', '24').done(),
    feature('Power', 'arithmetic').supported().pure().eval('power.pow1', 'Power[x, 1]', 'x').eval('power.square', '2^3', '8').done(),
    feature('Subtract', 'arithmetic').supported().pure().eval('subtract.basic', 'Subtract[5, 2]', '3').done(),
    feature('Divide', 'arithmetic')
        .supported()
        .pure()
        .eval('divide.basic', 'Divide[6, 2]', '3')
        .eval('divide.rational', '1/3 + 1/3 + 1/3', '1')
        .done(),
    feature('Factorial', 'arithmetic').supported().pure().eval('factorial.5', '5!', '120').done(),
    feature('Sqrt', 'arithmetic').supported().pure().eval('sqrt.4', 'Sqrt[4]', '2').done(),
    feature('Abs', 'arithmetic').supported().pure().eval('abs.neg', 'Abs[-3]', '3').done(),
    feature('Max', 'arithmetic').unsupported().pure().gap('max.3', 'Max[1, 3, 2]', { expected: '3' }).done(),
    feature('Floor', 'arithmetic').unsupported().pure().gap('floor.2_7', 'Floor[2.7]', { expected: '2' }).done(),
    feature('ArithCanonical', 'arithmetic')
        .supported()
        .pure()
        .notes('identity folds x+0 / 1*x / x^0 / like powers')
        .eval('arith.x_plus_0', 'x + 0', 'x')
        .eval('arith.one_times_x', '1 * x', 'x')
        .eval('arith.x_pow_0', 'x^0', '1')
        .eval('arith.pow_combine', 'x^2 * x^3', 'x^5')
        .done(),
    feature('Min', 'arithmetic').unsupported().pure().gap('min.3', 'Min[3, 1, 2]', { expected: '1' }).done(),
    feature('Sign', 'arithmetic').unsupported().pure().gap('sign.neg', 'Sign[-3]', { expected: '-1' }).done(),
    feature('Round', 'arithmetic').unsupported().pure().gap('round.2_5', 'Round[2.5]', { expected: '2' }).done(),
    feature('Ceiling', 'arithmetic').unsupported().pure().gap('ceiling.2_1', 'Ceiling[2.1]', { expected: '3' }).done(),
    feature('IndeterminateForms', 'arithmetic')
        .unsupported('SILENT WRONG: 0/0→0; Infinity-Infinity→0; 0^0→1 (MMA expects Indeterminate)')
        .pure()
        .gap('indet.0over0', '0/0', { expected: 'Indeterminate', notes: 'currently 0' })
        .gap('indet.inf_minus_inf', 'Infinity - Infinity', { expected: 'Indeterminate', notes: 'currently 0' })
        .gap('indet.0pow0', '0^0', { expected: 'Indeterminate', notes: 'currently 1' })
        .done(),
    feature('CubeRootPow', 'arithmetic')
        .unsupported('SILENT WRONG parse/precedence: (-8)^(1/3) → -8^1/3')
        .pure()
        .gap('cuberoot.neg8', '(-8)^(1/3)', { expected: '-2', notes: 'currently -8^1/3' })
        .done(),
    feature('SqrtRational', 'arithmetic').supported().pure().eval('sqrt.9_4', 'Sqrt[9/4]', '3/2').done(),
    feature('RationalAdd', 'arithmetic').supported().pure().eval('rational.add', '1/2 + 1/3', '5/6').done(),
    feature('CubeRoot', 'arithmetic').unsupported().pure().gap('cuberoot.m8', 'CubeRoot[-8]', { expected: '-2' }).done(),
    feature('Surd', 'arithmetic').unsupported().pure().gap('surd.m8_3', 'Surd[-8, 3]', { expected: '-2' }).done(),
    feature('Clip', 'arithmetic')
        .unsupported()
        .pure()
        .gap('clip.hi', 'Clip[5, {0, 1}]', { expected: '1' })
        .gap('clip.lo', 'Clip[-1, {0, 1}]', { expected: '0' })
        .done(),
    feature('Rescale', 'arithmetic').unsupported().pure().gap('rescale.mid', 'Rescale[0.5, {0, 1}, {-1, 1}]', { expected: '0' }).done(),
];
