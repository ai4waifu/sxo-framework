import { feature } from '@sxo/harness';

export const comparisonFeatures = [
    feature('eq', 'comparison').supported().pure().eval('eq.true', '3 == 3', 'true').done(),
    feature('ne', 'comparison')
        .partial('scalar ~= OK; vector [1,2]~=[1,3] stays Unequal head not elementwise mask')
        .pure()
        .eval('ne.true', '3 ~= 2', 'true')
        .eval('ne.false', '1 ~= 1', 'false')
        .gap('ne.vec', '[1, 2] ~= [1, 3]', { expected: '[0, 1]', notes: 'currently Unequal([1, 2], [1, 3])' })
        .done(),
    feature('le', 'comparison')
        .partial('scalar <= OK; vector / functional le(…) unevaluated')
        .pure()
        .eval('le.true', '2 <= 3', 'true')
        .gap('le.vec', '[1, 2] <= [1, 3]', { expected: '[1, 1]' })
        .gap('le.fn', 'le(1, 2)', { expected: 'true' })
        .done(),
    feature('ge', 'comparison')
        .partial('scalar >= OK; vector / functional ge(…) unevaluated')
        .pure()
        .eval('ge.true', '1 >= 1', 'true')
        .eval('ge.false', '1 >= 2', 'false')
        .gap('ge.vec', '[1, 2, 3] >= 2', { expected: '[0, 1, 1]' })
        .gap('ge.fn', 'ge(2, 2)', { expected: 'true' })
        .done(),
    feature('gt', 'comparison').supported().pure().eval('gt.true', '3 > 2', 'true').done(),
    feature('elementwise_compare', 'comparison')
        .unsupported('vectorized > / == become Greater/Equal heads, not logical masks')
        .pure()
        .gap('gt.vec', '[1, 2, 3] > 2', { expected: '[0, 0, 1]', notes: 'currently Greater([1, 2, 3], 2)' })
        .done(),
    feature('isequal', 'comparison').unsupported().pure().gap('isequal.vec', 'isequal([1, 2], [1, 2])', { expected: '1' }).done(),
    feature('lt_chain', 'comparison')
        .partial('scalar 1<2<3 via Athena compare chain; elementwise vector Less still open')
        .pure()
        .eval('ltchain.123', '1 < 2 < 3', 'true')
        .gap('ltchain.vec', '[1, 2, 3] < 2', { expected: '[1, 0, 0]' })
        .done(),
    feature('eq_fn', 'comparison')
        .unsupported('functional eq([1,2],[1,2]) unevaluated; true==1 stays Equal head')
        .pure()
        .gap('eq.fn_vec', 'eq([1, 2], [1, 2])', { expected: '[1, 1]' })
        .gap('eq.true_num', 'true == 1', { expected: '1', notes: 'currently Equal(true, 1)' })
        .done(),
    feature('isequaln', 'comparison').unsupported().pure().gap('isequaln.nan', 'isequaln([NaN], [NaN])', { expected: '1' }).done(),
    feature('nan_eq', 'comparison')
        .unsupported('NaN==NaN / NaN~=NaN stay Equal/Unequal heads (IEEE unmet)')
        .pure()
        .gap('nan.eq', 'NaN == NaN', { expected: '0', notes: 'currently Equal(NaN, NaN)' })
        .gap('nan.ne', 'NaN ~= NaN', { expected: '1', notes: 'currently Unequal(NaN, NaN)' })
        .done(),
];
