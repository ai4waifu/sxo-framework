import { feature } from '@sxo/harness';

export const logicFeatures = [
    feature('And', 'logic')
        .partial('short-circuit `And` on tested boolean and equality forms')
        .pure()
        .eval('and.equal', 'And[1 == 1, 2 == 2]', 'True')
        .eval('and.bool_atoms', 'And[True, False]', 'False')
        .done(),
    feature('Or', 'logic')
        .partial('short-circuit `Or` on tested boolean and equality forms')
        .pure()
        .eval('or.equal', 'Or[1 == 2, 2 == 2]', 'True')
        .eval('or.bool_atoms', 'Or[False, True]', 'True')
        .done(),
    feature('Not', 'logic')
        .partial('`Not` on tested boolean and equality forms')
        .pure()
        .eval('not.equal', 'Not[1 == 2]', 'True')
        .eval('not.true', 'Not[True]', 'False')
        .done(),
    feature('If', 'logic').partial('short-circuit `If` on tested boolean conditions').pure().eval('if.true', 'If[1 == 1, 7, 8]', '7').done(),
    feature('Which', 'logic')
        .partial('tested `Which` branch selection only')
        .pure()
        .eval('which.basic', 'Which[False, 1, True, 2]', '2')
        .done(),
    feature('Boole', 'logic')
        .partial('`Boole` on tested boolean predicates only')
        .pure()
        .eval('boole.true', 'Boole[True]', '1')
        .eval('boole.pred', 'Boole[2 > 1]', '1')
        .eval('boole.false', 'Boole[False]', '0')
        .done(),
    feature('Xor', 'logic')
        .partial('`Xor` on tested boolean atoms only')
        .pure()
        .eval('xor.tf', 'Xor[True, False]', 'True')
        .eval('xor.tt', 'Xor[True, True]', 'False')
        .done(),
    feature('Implies', 'logic')
        .partial('`Implies` on tested boolean atoms only')
        .pure()
        .eval('implies.tf', 'Implies[True, False]', 'False')
        .eval('implies.ff', 'Implies[False, False]', 'True')
        .done(),
];
