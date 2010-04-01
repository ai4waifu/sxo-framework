import { feature } from '@sxo/harness';

export const controlFeatures = [
    feature('Catch', 'control').unsupported().pure().gap('catch.throw', 'Catch[Throw[2]]', { expected: '2' }).done(),
    feature('Do', 'control')
        .supported()
        .stateful()
        .notes('Do lowers to CountedLoop then Null')
        .eval('do.count', 'Do[1, {3}]', 'Null')
        .eval('do.binder', 'Do[i, {i, 3}]', 'Null')
        .done(),
    feature('While', 'control')
        .supported()
        .stateful()
        .notes('While lowers to LoopWhile')
        .eval('while.false', 'While[False, 1]', 'Null')
        .done(),
    feature('For', 'control')
        .unsupported('oak error node on For[i=1,i<3,i++,i]')
        .stateful()
        .gap('for.basic', 'For[i = 1, i < 3, i++, i]', { expected: 'Null' })
        .done(),
];
