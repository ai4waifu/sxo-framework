import { feature } from '@sxo/harness';

export const comparisonFeatures = [
    feature('Equal', 'comparison')
        .partial('scalar and elementwise exact equality on tested forms')
        .pure()
        .eval('equal.true', '2 == 2', 'True')
        .eval('equal.list_broadcast', '{1, 2} == {1, 3}', '{True, False}')
        .done(),
    feature('Unequal', 'comparison')
        .partial('scalar exact inequality on tested forms only')
        .pure()
        .eval('unequal.true', '2 != 3', 'True')
        .eval('unequal.list_broadcast', '{1, 2} != {1, 3}', '{False, True}')
        .done(),
    feature('Less', 'comparison')
        .partial('scalar and elementwise `<` / `Less` on tested exact integers')
        .pure()
        .eval('less.infix', '2 < 3', 'True')
        .eval('less.head', 'Less[2, 3]', 'True')
        .eval('less.list_broadcast', '{1, 2, 3} < 2', '{True, False, False}')
        .done(),
    feature('Greater', 'comparison').partial('scalar `>` on tested exact integers only').pure().eval('greater.infix', '3 > 2', 'True').done(),
    feature('LessEqual', 'comparison')
        .partial('binary and n-ary `LessEqual` on exact integer chains')
        .pure()
        .eval('le.infix', '2 <= 3', 'True')
        .eval('le.eq', 'LessEqual[1, 1]', 'True')
        .eval('le.chain', 'LessEqual[1, 2, 3]', 'True')
        .done(),
    feature('GreaterEqual', 'comparison')
        .partial('binary and n-ary `GreaterEqual` on exact integer chains')
        .pure()
        .eval('ge.infix', '3 >= 2', 'True')
        .eval('ge.eq', 'GreaterEqual[2, 2]', 'True')
        .eval('ge.inequality', 'Inequality[1, Less, 2, Less, 3]', 'True')
        .done(),
    feature('SameQ', 'comparison')
        .partial('infix `===` and head-form `SameQ` structural identity on tested forms')
        .pure()
        .eval('sameq.num_infix', '1 === 1', 'True')
        .eval('sameq.sym_infix', 'x === x', 'True')
        .eval('sameq.head', 'SameQ[1, 1]', 'True')
        .done(),
    feature('UnsameQ', 'comparison')
        .partial('head-form `UnsameQ` structural inequality on tested literals')
        .pure()
        .eval('unsameq.12', 'UnsameQ[1, 2]', 'True')
        .eval('unsameq.11', 'UnsameQ[1, 1]', 'False')
        .done(),
    feature('InequalityChain', 'comparison')
        .partial('same-op and mixed infix chains plus head-form `Inequality` on exact integers')
        .pure()
        .notes('mixed infix rewrites to `And` of pairwise compares at compile time')
        .eval('ineq.lt_chain', '1 < 2 < 3', 'True')
        .eval('ineq.mixed', '1 < 3 > 2', 'True')
        .eval('ineq.head', 'Inequality[1, Less, 2, Less, 3]', 'True')
        .done(),
    feature('UnsameQInfix', 'comparison')
        .partial('numeric 1=!=2 → 1; symbols lower to Unequal not UnsameQ')
        .pure()
        .eval('unsameq.num_infix', '1 =!= 2', 'True')
        .gap('unsameq.sym_infix', 'x =!= x', { expected: 'False', notes: 'currently Unequal[x, x]' })
        .done(),
];
