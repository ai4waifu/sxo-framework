import { feature } from '@sxo/harness';

export const patternFeatures = [
    feature('Blank', 'pattern')
        .supported()
        .pure()
        .notes('typed Blank[Integer] for MatchQ/Cases')
        .eval('blank.matchq', 'MatchQ[1, _Integer]', 'True')
        .eval('blank.cases', 'Cases[{1, a, 2}, _Integer]', '{1, 2}')
        .done(),
    feature('MatchQ', 'pattern')
        .supported()
        .pure()
        .notes('ControlPlan::Match with typed Blank[Integer]')
        .eval('matchq.integer', 'MatchQ[1, _Integer]', 'True')
        .done(),
    feature('Condition', 'pattern')
        .unsupported('Blank stripped: Condition[x_,x>0] → Condition[x, Greater[x, 0]]')
        .pure()
        .gap('condition.pattern', 'MatchQ[2, x_ /; x > 0]', { expected: 'True' })
        .done(),
    feature('PatternTest', 'pattern')
        .unsupported('PatternTest not lowered; Cases stays Cases[list, Blank[], NumberQ]')
        .pure()
        .gap('patterntest.numberq', 'Cases[{1, a, 2}, _?NumberQ]', {
            expected: '{1, 2}',
            notes: 'unevaluated Cases with Blank[] + NumberQ args; not a silent strip to NumberQ',
        })
        .done(),
    feature('BlankSequence', 'pattern')
        .unsupported('MatchQ[f[1,2],f[__]] evaluates False (BlankSequence not matched)')
        .pure()
        .gap('blankseq.match', 'MatchQ[f[1, 2], f[__]]', {
            expected: 'True',
            notes: 'returns False; BlankSequence form is preserved (not stripped to f[])',
        })
        .done(),
    feature('BlankNullSequence', 'pattern')
        .unsupported('___ matching incomplete')
        .pure()
        .gap('blanknullseq.match', 'MatchQ[f[], f[___]]', { expected: 'True' })
        .done(),
    feature('PatternConditionDef', 'pattern')
        .unsupported('f[x_/;x>0]:=x oak error; f[x_?Positive]:=x; f[1] stays f[1]')
        .stateful()
        .gap('pattern.condition_def', 'f[x_ /; x > 0] := x; f[1]', { expected: '1', notes: 'oak error node' })
        .gap('pattern.patternTest_def', 'f[x_?Positive] := x; f[1]', { expected: '1', notes: 'currently f[1]' })
        .done(),
];
