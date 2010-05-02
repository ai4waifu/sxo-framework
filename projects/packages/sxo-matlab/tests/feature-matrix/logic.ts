import { feature } from '@sxo/harness';

export const logicFeatures = [
    feature('and', 'logic').supported().pure().notes('numeric short-circuit style && on 0/1').eval('and.10', '1 && 0', 'false').done(),
    feature('or', 'logic').supported().pure().eval('or.10', '1 || 0', 'true').done(),
    feature('not', 'logic').supported().pure().eval('not.1', '~1', 'false').eval('not.0', '~0', 'true').done(),
    feature('xor', 'logic').unsupported().pure().gap('xor.10', 'xor(1, 0)', { expected: '1' }).done(),
    feature('bitor_op', 'logic')
        .supported()
        .pure()
        .notes('`|` lowers to ElementwiseOr (not short-circuit Or)')
        .eval('bitor.scalar', '1 | 0', 'true')
        .eval('bitor.vec', '[1, 0] | [0, 1]', '[true, true]')
        .done(),
    feature('bitand_op', 'logic')
        .supported()
        .pure()
        .notes('`&` lowers to ElementwiseAnd (not short-circuit And)')
        .eval('bitand.scalar', '1 & 0', 'false')
        .eval('bitand.vec', '[1, 0] & [1, 1]', '[true, false]')
        .done(),
    feature('true_bitor', 'logic')
        .supported()
        .pure()
        .eval('true.bitor', 'true | false', 'true')
        .done(),
    feature('true_bitand', 'logic')
        .partial('true & false → false OK; true && false stays And(true,false) unevaluated')
        .pure()
        .eval('true.bitand', 'true & false', 'false')
        .gap('true.and_sc', 'true && false', { expected: '0', notes: 'currently And(true, false)' })
        .done(),
];
