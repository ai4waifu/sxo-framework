import { feature } from '@sxo/harness';

export const arithmeticFeatures = [
    feature('Plus', 'arithmetic').partial('exact integer `Plus` on tested forms only').pure().eval('plus.basic', '1 + 2 * 3', '7').eval('plus.nary', 'Plus[1, 2, 3]', '6').done(),
    feature('Times', 'arithmetic')
        .partial('exact integer `Times` and typed `ComplexExact` Hadamard on tested forms')
        .pure()
        .eval('times.nary', 'Times[2, 3, 4]', '24')
        .eval('times.complex_hadamard', '{{1 + I, 2}, {3, 4}}*{{1, I}, {0, 1}}', '{{1 + I, 2*I}, {0, 4}}')
        .done(),
    feature('Power', 'arithmetic')
        .partial('symbolic `Power[x,1]` and exact integer powers on tested forms. `ComplexExact` `1×1` matrix OK')
        .pure()
        .eval('power.pow1', 'Power[x, 1]', 'x')
        .eval('power.square', '2^3', '8')
        .eval('power.complex', '{{1 + I}}^2', '2*I')
        .done(),
    feature('Subtract', 'arithmetic').partial('exact integer `Subtract` on tested forms only').pure().eval('subtract.basic', 'Subtract[5, 2]', '3').done(),
    feature('Divide', 'arithmetic')
        .partial('exact rational divide on tested forms only')
        .pure()
        .eval('divide.basic', 'Divide[6, 2]', '3')
        .eval('divide.rational', '1/3 + 1/3 + 1/3', '1')
        .done(),
    feature('Factorial', 'arithmetic').partial('exact integer factorial on tested forms only').pure().eval('factorial.5', '5!', '120').done(),
    feature('Sqrt', 'arithmetic').partial('exact integer square root on tested forms only').pure().eval('sqrt.4', 'Sqrt[4]', '2').done(),
    feature('Abs', 'arithmetic').partial('exact integer `Abs` on tested forms only').pure().eval('abs.neg', 'Abs[-3]', '3').done(),
    feature('Max', 'arithmetic').unsupported().pure().gap('max.3', 'Max[1, 3, 2]', { expected: '3' }).done(),
    feature('Floor', 'arithmetic').unsupported().pure().gap('floor.2_7', 'Floor[2.7]', { expected: '2' }).done(),
    feature('ArithCanonical', 'arithmetic')
        .partial('tested identity folds `x+0` / `1*x` / `x^0` / like powers only')
        .pure()
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
        .partial('tested singular forms fold to `Indeterminate` via Athena domain rules')
        .pure()
        .eval('indet.0over0', '0/0', 'Indeterminate')
        .eval('indet.inf_minus_inf', 'Infinity - Infinity', 'Indeterminate')
        .eval('indet.0pow0', '0^0', 'Indeterminate')
        .eval('indet.div0_cancel', '(1/0)-(1/0)', 'Indeterminate')
        .done(),
    feature('CubeRootPow', 'arithmetic')
        .partial('exact integer cube root of negative base on tested pin only')
        .pure()
        .eval('cuberoot.neg8', '(-8)^(1/3)', '-2')
        .done(),
    feature('SqrtRational', 'arithmetic').partial('exact rational square root on tested forms only').pure().eval('sqrt.9_4', 'Sqrt[9/4]', '3/2').done(),
    feature('RationalAdd', 'arithmetic').partial('exact rational addition on tested forms only').pure().eval('rational.add', '1/2 + 1/3', '5/6').done(),
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
