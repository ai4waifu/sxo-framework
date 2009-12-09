import { feature } from '@sxo/harness';

export const ruleFeatures = [
    feature('Rule', 'rule')
        .partial('head ReplaceAll[expr, lhs->rhs] works for simple symbols; RuleDelayed / infix /. fragile')
        .pure()
        .eval('rule.replaceall_pow', 'ReplaceAll[x^2, x -> 3]', '9')
        .gap('rule.replaceall_named', 'ReplaceAll[x^2, Rule[x, 3]]', { expected: '9', notes: 'Rule[…] may flatten args' })
        .done(),
    feature('RuleDelayed', 'rule').planned().pure().gap('ruledelayed.basic', 'x /. x :> 1 + 1', { expected: '2' }).done(),
    feature('Replace', 'rule').unsupported().pure().gap('replace.basic', 'Replace[a, a -> 1]', { expected: '1' }).done(),
    feature('ReplaceAll', 'rule')
        .partial(
            'head form works for symbol and list-of-rules; infix /. often oak error; Hold args evaluate first so ReplaceAll[Hold[1+1],1->2] → Hold[2] (cascades Hold silent wrong)',
        )
        .pure()
        .eval('replaceall.head', 'ReplaceAll[x, x -> 3]', '3')
        .eval('replaceall.list_rules', 'ReplaceAll[{a, b}, {a -> 1, b -> 2}]', '{1, 2}')
        .gap('replaceall.infix', 'x^2 /. x -> 3', { expected: '9' })
        .gap('replaceall.into_hold', 'ReplaceAll[Hold[1 + 1], 1 -> 2]', {
            expected: 'Hold[2 + 2]',
            notes: 'currently Hold[2] after Hold evaluates Plus',
        })
        .done(),
    feature('ReplaceList', 'rule')
        .unsupported('Pattern stripped; may return RHS vars like {a,b,c} instead of matches')
        .pure()
        .gap('replacelist.pair', 'ReplaceList[{1, 2}, {a_, b_} :> a + b]', { expected: '{3}' })
        .done(),
    feature('FilterRules', 'rule')
        .unsupported()
        .pure()
        .gap('filterrules.a', 'FilterRules[{a -> 1, b -> 2}, {a}]', { expected: '{a -> 1}' })
        .done(),
];
