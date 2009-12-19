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
        .unsupported('SILENT WRONG: Blank stripped — MatchQ[1,_Integer]→MatchQ[1,Integer]; MatchQ[f[1],f[_]]→MatchQ[f[1],f[]]')
        .pure()
        .gap('matchq.integer', 'MatchQ[1, _Integer]', { expected: 'True', notes: 'currently MatchQ[1, Integer]' })
        .done(),
    feature('Condition', 'pattern')
        .unsupported('Blank stripped: Condition[x_,x>0] → Condition[x, Greater[x, 0]]')
        .pure()
        .gap('condition.pattern', 'MatchQ[2, x_ /; x > 0]', { expected: 'True' })
        .done(),
    feature('PatternTest', 'pattern')
        .unsupported('SILENT WRONG: Cases[{1,a,2},_?NumberQ] → NumberQ (same Blank/? strip as MatchQ)')
        .pure()
        .gap('patterntest.numberq', 'Cases[{1, a, 2}, _?NumberQ]', { expected: '{1, 2}', notes: 'currently returns NumberQ' })
        .done(),
    feature('BlankSequence', 'pattern')
        .unsupported('SILENT WRONG: MatchQ[f[1,2],f[__]] → MatchQ[f[1,2],f[]] (__ stripped)')
        .pure()
        .gap('blankseq.match', 'MatchQ[f[1, 2], f[__]]', { expected: 'True', notes: 'currently MatchQ[f[1, 2], f[]]' })
        .done(),
    feature('BlankNullSequence', 'pattern')
        .unsupported('SILENT WRONG: ___ stripped to empty args')
        .pure()
        .gap('blanknullseq.match', 'MatchQ[f[], f[___]]', { expected: 'True', notes: 'currently MatchQ[f[], f[]]' })
        .done(),
    feature('PatternConditionDef', 'pattern')
        .unsupported('f[x_/;x>0]:=x oak error; f[x_?Positive]:=x; f[1] stays f[1]')
        .stateful()
        .gap('pattern.condition_def', 'f[x_ /; x > 0] := x; f[1]', { expected: '1', notes: 'oak error node' })
        .gap('pattern.patternTest_def', 'f[x_?Positive] := x; f[1]', { expected: '1', notes: 'currently f[1]' })
        .done(),
];
