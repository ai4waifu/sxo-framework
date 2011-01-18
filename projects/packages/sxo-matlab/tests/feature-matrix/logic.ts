import { feature } from '@sxo/harness';

export const logicFeatures = [
    feature('and', 'logic').partial('scalar short-circuit `&&` on tested forms').pure().eval('and.10', '1 && 0', 'false').done(),
    feature('or', 'logic').partial('scalar short-circuit `||` on tested forms').pure().eval('or.10', '1 || 0', 'true').done(),
    feature('not', 'logic').partial('scalar `~` on tested `0`/`1` only').pure().eval('not.1', '~1', 'false').eval('not.0', '~0', 'true').done(),
    feature('xor', 'logic').unsupported().pure().gap('xor.10', 'xor(1, 0)', { expected: '1' }).done(),
    feature('bitor_op', 'logic')
        .partial('`|` elementwise or on tested scalar and vector forms')
        .pure()
        .eval('bitor.scalar', '1 | 0', 'true')
        .eval('bitor.vec', '[1, 0] | [0, 1]', '[true, true]')
        .done(),
    feature('bitand_op', 'logic')
        .partial('`&` elementwise and on tested scalar and vector forms')
        .pure()
        .eval('bitand.scalar', '1 & 0', 'false')
        .eval('bitand.vec', '[1, 0] & [1, 1]', '[true, false]')
        .done(),
    feature('true_bitor', 'logic').partial('boolean atom `|` on tested forms only').pure().eval('true.bitor', 'true | false', 'true').done(),
    feature('true_bitand', 'logic')
        .partial('boolean atom `&` and short-circuit `&&` on tested forms')
        .pure()
        .eval('true.bitand', 'true & false', 'false')
        .eval('true.and_sc', 'true && false', 'false')
        .done(),
];
