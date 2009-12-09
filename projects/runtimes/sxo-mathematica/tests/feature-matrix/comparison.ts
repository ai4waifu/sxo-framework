import { feature } from '@sxo/harness';

export const comparisonFeatures = [
    feature('Equal', 'comparison').supported().pure().eval('equal.true', '2 == 2', 'True').done(),
    feature('Unequal', 'comparison').supported().pure().eval('unequal.true', '2 != 3', 'True').done(),
    feature('Less', 'comparison').supported().pure().eval('less.infix', '2 < 3', 'True').eval('less.head', 'Less[2, 3]', 'True').done(),
    feature('Greater', 'comparison').supported().pure().eval('greater.infix', '3 > 2', 'True').done(),
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
        .partial('SILENT WRONG: === lowers to Equal for symbols (x===x → Equal[x,x]); SameQ[1,1] unevaluated; numeric 1===1 → 1')
        .pure()
        .eval('sameq.num_infix', '1 === 1', 'True')
        .gap('sameq.head', 'SameQ[1, 1]', { expected: 'True' })
        .gap('sameq.sym_infix', 'x === x', { expected: 'True', notes: 'currently Equal[x, x]' })
        .done(),
    feature('UnsameQ', 'comparison').unsupported().pure().gap('unsameq.12', 'UnsameQ[1, 2]', { expected: 'True' }).done(),
    feature('InequalityChain', 'comparison')
        .supported()
        .pure()
        .notes('nested relational ops evaluate via Athena compare-chain flattening')
        .eval('ineq.lt_chain', '1 < 2 < 3', 'True')
        .eval('ineq.mixed', '1 < 3 > 2', 'True')
        .done(),
    feature('UnsameQInfix', 'comparison')
        .partial('numeric 1=!=2 → 1; symbols lower to Unequal not UnsameQ')
        .pure()
        .eval('unsameq.num_infix', '1 =!= 2', 'True')
        .gap('unsameq.sym_infix', 'x =!= x', { expected: 'False', notes: 'currently Unequal[x, x]' })
        .done(),
];
