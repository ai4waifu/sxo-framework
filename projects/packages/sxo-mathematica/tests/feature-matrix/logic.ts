import { feature } from '@sxo/harness';

export const logicFeatures = [
    feature('And', 'logic')
        .supported()
        .pure()
        .notes('typed Boolean atoms; And over Equal and True/False')
        .eval('and.equal', 'And[1 == 1, 2 == 2]', 'True')
        .eval('and.bool_atoms', 'And[True, False]', 'False')
        .done(),
    feature('Or', 'logic')
        .supported()
        .pure()
        .eval('or.equal', 'Or[1 == 2, 2 == 2]', 'True')
        .eval('or.bool_atoms', 'Or[False, True]', 'True')
        .done(),
    feature('Not', 'logic').supported().pure().eval('not.equal', 'Not[1 == 2]', 'True').eval('not.true', 'Not[True]', 'False').done(),
    feature('If', 'logic')
        .supported()
        .pure()
        .notes('short-circuit; non-boolean condition is structured diagnostic at Athena')
        .eval('if.true', 'If[1 == 1, 7, 8]', '7')
        .done(),
    feature('Which', 'logic').supported().pure().eval('which.basic', 'Which[False, 1, True, 2]', '2').done(),
    feature('Boole', 'logic')
        .supported()
        .pure()
        .notes('Boole lowers to Branch → 1/0')
        .eval('boole.true', 'Boole[True]', '1')
        .eval('boole.pred', 'Boole[2 > 1]', '1')
        .eval('boole.false', 'Boole[False]', '0')
        .done(),
    feature('Xor', 'logic')
        .supported()
        .pure()
        .notes('Xor lowers to nested Branch')
        .eval('xor.tf', 'Xor[True, False]', 'True')
        .eval('xor.tt', 'Xor[True, True]', 'False')
        .done(),
    feature('Implies', 'logic')
        .supported()
        .pure()
        .notes('Implies lowers to Branch(antecedent, consequent, True)')
        .eval('implies.tf', 'Implies[True, False]', 'False')
        .eval('implies.ff', 'Implies[False, False]', 'True')
        .done(),
];
