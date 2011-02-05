import { feature } from '@sxo/harness';

export const controlFeatures = [
    feature('if', 'control')
        .partial('tested `if`/`else` on scalar conditions only')
        .pure()
        .eval('if.else', 'if 1, 2, else, 3, end', '2')
        .done(),
    feature('for', 'control')
        .partial('`for i=1:n` last value and tested accumulator in one Session')
        .stateful()
        .eval('for.last', 'for i=1:3, i, end', '3')
        .eval('for.sum', 's=0; for i=1:3, s=s+i; end; s', '6')
        .done(),
    feature('while', 'control').partial('`while 0` skips body on tested form').stateful().eval('while.false', 'while 0, 1, end', '[]').done(),
    feature('switch', 'control')
        .partial('tested `switch`/`case`/`otherwise` without fall-through')
        .pure()
        .eval('switch.case1', 'switch 1, case 1, 2, otherwise, 3, end', '2')
        .eval('switch.otherwise', 'switch 2, case 1, 2, otherwise, 3, end', '3')
        .done(),
    feature('try_catch', 'control')
        .partial('tested `try`/`catch` success and `error` paths only')
        .pure()
        .eval('try.catch', "try, error('e'), catch, 1, end", '1')
        .eval('try.no_error', 'try, 2, catch, 3, end', '2')
        .done(),
    feature('assert', 'control').unsupported().pure().gap('assert.true', 'assert(1)', { expected: '...' }).done(),
    feature('parfor', 'control')
        .unsupported('parse keeps Parfor Form via oak Statement::Parfor; parallel for runtime still open')
        .stateful()
        .gap('parfor.strip', 'parfor i=1:2, i, end', {
            expected: '2',
            notes: 'Form Parfor[i, 1:2, i]; eval Reject (was juxta parse error / silent strip)',
        })
        .done(),
];
