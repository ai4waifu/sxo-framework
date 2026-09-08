import { feature } from '@sxo/harness';

export const holdFeatures = [
    feature('Hold', 'hold')
        .supported()
        .unevaluated()
        .pure()
        .notes('HoldAll args preserved on evaluate (no autoSimplify fold through Hold)')
        .eval('hold.plus', 'Hold[1 + 1]', 'Hold[1 + 1]')
        .done(),
    feature('HoldForm', 'hold')
        .supported()
        .unevaluated()
        .pure()
        .notes('dialect maps HoldForm → neutral Hold; held Add preserved')
        .eval('holdform.plus', 'HoldForm[1 + 1]', 'Hold[1 + 1]')
        .done(),
    feature('Evaluate', 'hold')
        .supported()
        .pure()
        .notes('Evaluate unwraps Hold/HoldForm/HoldComplete then evaluates')
        .eval('evaluate.hold', 'Evaluate[Hold[1 + 1]]', '2')
        .done(),
    feature('ReleaseHold', 'hold')
        .supported()
        .pure()
        .notes('ReleaseHold unwraps Hold/HoldForm/HoldComplete then evaluates')
        .eval('releasehold.plus', 'ReleaseHold[Hold[1 + 1]]', '2')
        .done(),
    feature('Unevaluated', 'hold')
        .supported()
        .unevaluated()
        .pure()
        .notes('Athena Unevaluated CaptureAsTerm; Evaluate unwraps')
        .eval('unevaluated.plus', 'Unevaluated[1 + 1]', 'Unevaluated[1 + 1]')
        .eval('evaluate.unevaluated', 'Evaluate[Unevaluated[1 + 1]]', '2')
        .done(),
    feature('HoldComplete', 'hold')
        .supported()
        .unevaluated()
        .pure()
        .notes('Athena HoldComplete CaptureAsTerm; ReleaseHold/Evaluate unwrap')
        .eval('holdcomplete.plus', 'HoldComplete[1 + 1]', 'HoldComplete[1 + 1]')
        .eval('releasehold.holdcomplete', 'ReleaseHold[HoldComplete[1 + 1]]', '2')
        .done(),
    feature('Inactive', 'hold')
        .partial('Inactive[Plus][1,2] retained; Inactivate[1+2] forces arg first → Inactivate[3]')
        .unevaluated()
        .eval('inactive.plus', 'Inactive[Plus][1, 2]', 'Inactive[Plus][1, 2]')
        .gap('inactivate.plus', 'Inactivate[1 + 2]', { expected: 'Inactive[Plus][1, 2]', notes: 'currently Inactivate[3]' })
        .done(),
    feature('Activate', 'hold').unsupported().pure().gap('activate.plus', 'Activate[Inactive[Plus][1, 2]]', { expected: '3' }).done(),
];
