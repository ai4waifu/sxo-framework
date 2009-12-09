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
        .unsupported('SILENT WRONG: Boole[True] → Boole[] (True stripped); Boole[2>1] → Boole[1] (no 0/1 coerce)')
        .pure()
        .gap('boole.true', 'Boole[True]', { expected: '1', notes: 'currently Boole[]' })
        .gap('boole.pred', 'Boole[2 > 1]', { expected: '1', notes: 'currently Boole[1]' })
        .done(),
    feature('Xor', 'logic')
        .unsupported('SILENT WRONG: Xor[True,False] → Xor[] (True/False stripped)')
        .pure()
        .gap('xor.tf', 'Xor[True, False]', { expected: 'True', notes: 'currently Xor[]' })
        .done(),
    feature('Implies', 'logic')
        .unsupported('SILENT WRONG: Implies[True,False] → Implies[]')
        .pure()
        .gap('implies.tf', 'Implies[True, False]', { expected: 'False', notes: 'currently Implies[]' })
        .done(),
];
