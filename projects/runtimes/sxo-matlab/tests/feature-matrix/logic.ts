import { feature } from '@sxo/harness';

export const logicFeatures = [
    feature('and', 'logic').supported().pure().notes('numeric short-circuit style && on 0/1').eval('and.10', '1 && 0', 'false').done(),
    feature('or', 'logic').supported().pure().eval('or.10', '1 || 0', 'true').done(),
    feature('not', 'logic').supported().pure().eval('not.1', '~1', 'false').eval('not.0', '~0', 'true').done(),
    feature('xor', 'logic').unsupported().pure().gap('xor.10', 'xor(1, 0)', { expected: '1' }).done(),
    feature('bitor_op', 'logic')
        .unsupported('SILENT WRONG: 1|0 → 0 (expect 1); [1,0]|[0,1] → [0,1] (expect [1,1])')
        .pure()
        .gap('bitor.scalar', '1 | 0', { expected: '1', notes: 'currently 0' })
        .gap('bitor.vec', '[1, 0] | [0, 1]', { expected: '[1, 1]', notes: 'currently [0, 1]' })
        .done(),
    feature('bitand_op', 'logic')
        .partial('scalar 1&0 → false OK; vector [1,0]&[1,1] → [1,1] SILENT WRONG (expect [1,0])')
        .pure()
        .eval('bitand.scalar', '1 & 0', 'false')
        .gap('bitand.vec', '[1, 0] & [1, 1]', { expected: '[1, 0]', notes: 'currently [1, 1]' })
        .done(),
    feature('true_bitor', 'logic')
        .unsupported('SILENT WRONG: true | false → false (same bug as 1|0)')
        .pure()
        .gap('true.bitor', 'true | false', { expected: '1', notes: 'currently false' })
        .done(),
    feature('true_bitand', 'logic')
        .partial('true & false → false OK; true && false stays And(true,false) unevaluated')
        .pure()
        .eval('true.bitand', 'true & false', 'false')
        .gap('true.and_sc', 'true && false', { expected: '0', notes: 'currently And(true, false)' })
        .done(),
];
