import { feature } from '@sxo/harness';

export const holdFeatures = [
    feature('Hold', 'hold')
        .partial('evaluate preserves Hold; autoSimplify still folds held Add on pinned Athena')
        .unevaluated()
        .notes('HoldAll args preserved on evaluate; simplify-through-Hold needs a newer green Athena')
        .gap('hold.plus', 'Hold[1 + 1]', { expected: 'Hold[1 + 1]', notes: 'autoSimplify → Hold[2] on Athena 4e59b260' })
        .done(),
    feature('HoldForm', 'hold')
        .partial('dialect maps HoldForm → neutral Hold; same simplify caveat as Hold')
        .unevaluated()
        .gap('holdform.plus', 'HoldForm[1 + 1]', { expected: 'Hold[1 + 1]', notes: 'autoSimplify → Hold[2] on Athena 4e59b260' })
        .done(),
    feature('Evaluate', 'hold')
        .unsupported('Hold args already evaluated: Evaluate[Hold[1+1]] → Evaluate[Hold[2]]')
        .pure()
        .gap('evaluate.hold', 'Evaluate[Hold[1 + 1]]', { expected: '2' })
        .done(),
    feature('ReleaseHold', 'hold')
        .unsupported('Hold already forced: ReleaseHold[Hold[1+1]] → ReleaseHold[Hold[2]]')
        .pure()
        .gap('releasehold.plus', 'ReleaseHold[Hold[1 + 1]]', { expected: '2' })
        .done(),
    feature('Unevaluated', 'hold')
        .unsupported('SILENT WRONG: Unevaluated[1+1] → Unevaluated[2]')
        .pure()
        .gap('unevaluated.plus', 'Unevaluated[1 + 1]', { expected: 'Unevaluated[1 + 1]' })
        .done(),
    feature('HoldComplete', 'hold')
        .unsupported('SILENT WRONG: HoldComplete[1+1] → HoldComplete[2]')
        .unevaluated()
        .gap('holdcomplete.plus', 'HoldComplete[1 + 1]', { expected: 'HoldComplete[1 + 1]', notes: 'currently HoldComplete[2]' })
        .done(),
    feature('Inactive', 'hold')
        .partial('Inactive[Plus][1,2] retained; Inactivate[1+2] forces arg first → Inactivate[3]')
        .unevaluated()
        .eval('inactive.plus', 'Inactive[Plus][1, 2]', 'Inactive[Plus][1, 2]')
        .gap('inactivate.plus', 'Inactivate[1 + 2]', { expected: 'Inactive[Plus][1, 2]', notes: 'currently Inactivate[3]' })
        .done(),
    feature('Activate', 'hold').unsupported().pure().gap('activate.plus', 'Activate[Inactive[Plus][1, 2]]', { expected: '3' }).done(),
];
