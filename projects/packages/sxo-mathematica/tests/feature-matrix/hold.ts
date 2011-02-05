import { feature } from '@sxo/harness';

export const holdFeatures = [
    feature('Hold', 'hold')
        .partial('HoldAll args preserved on tested evaluate paths. Broader hold timing still open')
        .unevaluated()
        .pure()
        .eval('hold.plus', 'Hold[1 + 1]', 'Hold[1 + 1]')
        .done(),
    feature('HoldForm', 'hold')
        .partial('dialect maps `HoldForm` → neutral `Hold` on tested forms')
        .unevaluated()
        .pure()
        .eval('holdform.plus', 'HoldForm[1 + 1]', 'Hold[1 + 1]')
        .done(),
    feature('Evaluate', 'hold')
        .partial('`Evaluate` unwraps tested `Hold`/`HoldForm`/`HoldComplete` heads')
        .pure()
        .eval('evaluate.hold', 'Evaluate[Hold[1 + 1]]', '2')
        .done(),
    feature('ReleaseHold', 'hold')
        .partial('`ReleaseHold` unwraps tested `Hold`/`HoldForm`/`HoldComplete` heads')
        .pure()
        .eval('releasehold.plus', 'ReleaseHold[Hold[1 + 1]]', '2')
        .done(),
    feature('Unevaluated', 'hold')
        .partial('`Unevaluated` capture and `Evaluate` unwrap on tested forms')
        .unevaluated()
        .pure()
        .eval('unevaluated.plus', 'Unevaluated[1 + 1]', 'Unevaluated[1 + 1]')
        .eval('evaluate.unevaluated', 'Evaluate[Unevaluated[1 + 1]]', '2')
        .done(),
    feature('HoldComplete', 'hold')
        .partial('`HoldComplete` capture and unwrap on tested forms. `Flatten` does not enter')
        .unevaluated()
        .pure()
        .eval('holdcomplete.plus', 'HoldComplete[1 + 1]', 'HoldComplete[1 + 1]')
        .eval('releasehold.holdcomplete', 'ReleaseHold[HoldComplete[1 + 1]]', '2')
        .eval('flatten.holdcomplete', 'Flatten[HoldComplete[{{1, 2}, {3}}]]', 'Flatten[HoldComplete[{{1, 2}, {3}}]]')
        .done(),
    feature('Inactive', 'hold')
        .partial('Inactive[Plus][1,2] retained; Inactivate[1+2] forces arg first → Inactivate[3]')
        .unevaluated()
        .eval('inactive.plus', 'Inactive[Plus][1, 2]', 'Inactive[Plus][1, 2]')
        .gap('inactivate.plus', 'Inactivate[1 + 2]', { expected: 'Inactive[Plus][1, 2]', notes: 'currently Inactivate[3]' })
        .done(),
    feature('Activate', 'hold').unsupported().pure().gap('activate.plus', 'Activate[Inactive[Plus][1, 2]]', { expected: '3' }).done(),
];
