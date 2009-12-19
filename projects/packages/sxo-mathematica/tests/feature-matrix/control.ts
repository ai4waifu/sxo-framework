import { feature } from '@sxo/harness';

export const controlFeatures = [
    feature('Catch', 'control').unsupported().pure().gap('catch.throw', 'Catch[Throw[2]]', { expected: '2' }).done(),
    feature('Do', 'control')
        .unsupported('SILENT WRONG: Do[1,{3}] → {3}; Do[i,{i,3}] → {i, 3} (body stripped like Table)')
        .stateful()
        .gap('do.strip', 'Do[1, {3}]', { expected: 'Null', notes: 'currently returns {3}' })
        .done(),
    feature('While', 'control')
        .unsupported('SILENT WRONG: While[False,1] → 1 (body runs / False atom broken)')
        .stateful()
        .gap('while.false', 'While[False, 1]', { expected: 'Null', notes: 'currently returns 1' })
        .done(),
    feature('For', 'control')
        .unsupported('oak error node on For[i=1,i<3,i++,i]')
        .stateful()
        .gap('for.basic', 'For[i = 1, i < 3, i++, i]', { expected: 'Null' })
        .done(),
];
