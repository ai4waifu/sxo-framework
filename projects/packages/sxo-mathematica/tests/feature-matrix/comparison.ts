import { feature } from '@sxo/harness';

export const comparisonFeatures = [
    feature('Equal', 'comparison').partial('scalar exact equality on tested forms only').pure().eval('equal.true', '2 == 2', 'True').done(),
    feature('Unequal', 'comparison')
        .partial('scalar exact inequality on tested forms only')
        .pure()
        .eval('unequal.true', '2 != 3', 'True')
        .done(),
    feature('Less', 'comparison')
        .partial('scalar `<` / `Less` on tested exact integers only')
        .pure()
        .eval('less.infix', '2 < 3', 'True')
        .eval('less.head', 'Less[2, 3]', 'True')
        .done(),
    feature('Greater', 'comparison').partial('scalar `>` on tested exact integers only').pure().eval('greater.infix', '3 > 2', 'True').done(),
    feature('LessEqual', 'comparison')
        .partial('binary <= / LessEqual returns typed Boolean; n-ary LessEqual[1,2,3] unevaluated')
        .pure()
        .eval('le.infix', '2 <= 3', 'True')
        .eval('le.eq', 'LessEqual[1, 1]', 'True')
        .gap('le.chain', 'LessEqual[1, 2, 3]', { expected: 'True' })
        .done(),
    feature('GreaterEqual', 'comparison')
        .partial('binary >= / GreaterEqual returns typed Boolean; Inequality chain unevaluated')
        .pure()
        .eval('ge.infix', '3 >= 2', 'True')
        .eval('ge.eq', 'GreaterEqual[2, 2]', 'True')
        .gap('ge.inequality', 'Inequality[1, Less, 2, Less, 3]', { expected: 'True' })
        .done(),
    feature('SameQ', 'comparison')
        .partial('infix === works for numbers and symbols; head-form SameQ[1,1] unevaluated')
        .pure()
        .eval('sameq.num_infix', '1 === 1', 'True')
        .eval('sameq.sym_infix', 'x === x', 'True')
        .gap('sameq.head', 'SameQ[1, 1]', { expected: 'True', notes: 'stays SameQ[1, 1]' })
        .done(),
    feature('UnsameQ', 'comparison').unsupported().pure().gap('unsameq.12', 'UnsameQ[1, 2]', { expected: 'True' }).done(),
    feature('InequalityChain', 'comparison')
        .partial('same-op chains OK via Athena flatten; mixed ops still nest and need Inequality sugar')
        .pure()
        .notes('nested relational ops evaluate via Athena compare-chain flattening')
        .eval('ineq.lt_chain', '1 < 2 < 3', 'True')
        .gap('ineq.mixed', '1 < 3 > 2', { expected: 'True', notes: 'currently Greater[True, 2]' })
        .done(),
    feature('UnsameQInfix', 'comparison')
        .partial('numeric 1=!=2 → 1; symbols lower to Unequal not UnsameQ')
        .pure()
        .eval('unsameq.num_infix', '1 =!= 2', 'True')
        .gap('unsameq.sym_infix', 'x =!= x', { expected: 'False', notes: 'currently Unequal[x, x]' })
        .done(),
];
