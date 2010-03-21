import { feature } from '@sxo/harness';

export const controlFeatures = [
    feature('Catch', 'control').unsupported().pure().gap('catch.throw', 'Catch[Throw[2]]', { expected: '2' }).done(),
    feature('Do', 'control')
        .unsupported('unevaluated Do[1,{3}] (CountedLoop / Null return not wired for Do head)')
        .stateful()
        .gap('do.strip', 'Do[1, {3}]', {
            expected: 'Null',
            notes: 'stays Do[1, {3}]; no longer strips to {3}',
        })
        .done(),
    feature('While', 'control')
        .unsupported('unevaluated While[False,1] (LoopWhile surface not wired for While head)')
        .stateful()
        .gap('while.false', 'While[False, 1]', {
            expected: 'Null',
            notes: 'stays While[False, 1]; False atom preserved (no longer returns 1)',
        })
        .done(),
    feature('For', 'control')
        .unsupported('oak error node on For[i=1,i<3,i++,i]')
        .stateful()
        .gap('for.basic', 'For[i = 1, i < 3, i++, i]', { expected: 'Null' })
        .done(),
];
