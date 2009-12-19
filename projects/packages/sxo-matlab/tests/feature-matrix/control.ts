import { feature } from '@sxo/harness';

export const controlFeatures = [
    feature('if', 'control').supported().pure().eval('if.else', 'if 1, 2, else, 3, end', '2').done(),
    feature('for', 'control')
        .supported()
        .stateful()
        .notes('for i=1:n last value and compound accumulator via shared Session bindings')
        .eval('for.last', 'for i=1:3, i, end', '3')
        .eval('for.sum', 's=0; for i=1:3, s=s+i; end; s', '6')
        .done(),
    feature('while', 'control')
        .supported()
        .stateful()
        .notes('while 0 skips body; empty result renders as []')
        .eval('while.false', 'while 0, 1, end', '[]')
        .done(),
    feature('switch', 'control')
        .unsupported('SILENT WRONG: switch 1, case 1, 2, otherwise, 3, end → 3')
        .pure()
        .gap('switch.case1', 'switch 1, case 1, 2, otherwise, 3, end', { expected: '2', notes: 'currently returns 3' })
        .done(),
    feature('try_catch', 'control')
        .supported()
        .pure()
        .notes('oak Statement::Try → Athena Try[body, catch]; success and error paths')
        .eval('try.catch', "try, error('e'), catch, 1, end", '1')
        .eval('try.no_error', 'try, 2, catch, 3, end', '2')
        .done(),
    feature('assert', 'control').unsupported().pure().gap('assert.true', 'assert(1)', { expected: '...' }).done(),
    feature('parfor', 'control')
        .unsupported('SILENT WRONG: parfor i=1:2, i, end → i (same strip as for)')
        .stateful()
        .gap('parfor.strip', 'parfor i=1:2, i, end', { expected: '2', notes: 'currently returns i' })
        .done(),
];
