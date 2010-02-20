import { feature } from '@sxo/harness';

export const comparisonFeatures = [
    feature('eq', 'comparison').supported().pure().eval('eq.true', '3 == 3', 'true').done(),
    feature('ne', 'comparison')
        .partial('scalar ~= OK; vector Unequal returns boolean mask (not 0/1)')
        .pure()
        .eval('ne.true', '3 ~= 2', 'true')
        .eval('ne.false', '1 ~= 1', 'false')
        .eval('ne.vec', '[1, 2] ~= [1, 3]', '[false, true]')
        .done(),
    feature('le', 'comparison')
        .partial('scalar <= OK; vector mask OK; functional le(…) still open')
        .pure()
        .eval('le.true', '2 <= 3', 'true')
        .eval('le.vec', '[1, 2] <= [1, 3]', '[true, true]')
        .gap('le.fn', 'le(1, 2)', { expected: 'true' })
        .done(),
    feature('ge', 'comparison')
        .partial('scalar >= OK; vector mask OK; functional ge(…) still open')
        .pure()
        .eval('ge.true', '1 >= 1', 'true')
        .eval('ge.false', '1 >= 2', 'false')
        .eval('ge.vec', '[1, 2, 3] >= 2', '[false, true, true]')
        .gap('ge.fn', 'ge(2, 2)', { expected: 'true' })
        .done(),
    feature('gt', 'comparison').supported().pure().eval('gt.true', '3 > 2', 'true').done(),
    feature('elementwise_compare', 'comparison')
        .supported()
        .pure()
        .notes('vectorized relations return boolean masks rendered as true/false')
        .eval('gt.vec', '[1, 2, 3] > 2', '[false, false, true]')
        .done(),
    feature('isequal', 'comparison').unsupported().pure().gap('isequal.vec', 'isequal([1, 2], [1, 2])', { expected: '1' }).done(),
    feature('lt_chain', 'comparison')
        .supported()
        .pure()
        .notes('scalar chains and elementwise vector Less')
        .eval('ltchain.123', '1 < 2 < 3', 'true')
        .eval('ltchain.vec', '[1, 2, 3] < 2', '[true, false, false]')
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
