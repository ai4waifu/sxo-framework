import { feature } from '@sxo/harness';

export const controlFeatures = [
    feature('Catch', 'control').unsupported().pure().gap('catch.throw', 'Catch[Throw[2]]', { expected: '2' }).done(),
    feature('Do', 'control')
        .partial('`Do` counted loop on tested iterator forms only')
        .stateful()
        .eval('do.count', 'Do[1, {3}]', 'Null')
        .eval('do.binder', 'Do[i, {i, 3}]', 'Null')
        .done(),
    feature('While', 'control').partial('`While[False, …]` no-op on tested form').stateful().eval('while.false', 'While[False, 1]', 'Null').done(),
    feature('For', 'control')
        .unsupported('oak error node on For[i=1,i<3,i++,i]')
        .stateful()
        .gap('for.basic', 'For[i = 1, i < 3, i++, i]', { expected: 'Null' })
        .done(),
];
