import { feature } from '@sxo/harness';

export const predicatesFeatures = [
    feature('PossibleZeroQ', 'predicates').unsupported().pure().gap('possiblezeroq.0', 'PossibleZeroQ[0]', { expected: 'True' }).done(),
    feature('NumericQ', 'predicates').unsupported().pure().gap('numericq.1', 'NumericQ[1]', { expected: 'True' }).done(),
    feature('IntegerQ', 'predicates').unsupported().pure().gap('integerq.1', 'IntegerQ[1]', { expected: 'True' }).done(),
    feature('AtomQ', 'predicates').unsupported().pure().gap('atomq.1', 'AtomQ[1]', { expected: 'True' }).done(),
    feature('NumberQ', 'predicates').unsupported().pure().gap('numberq.12', 'NumberQ[1.2]', { expected: 'True' }).done(),
    feature('EvenQ', 'predicates').unsupported().pure().gap('evenq.2', 'EvenQ[2]', { expected: 'True' }).done(),
    feature('Positive', 'predicates').unsupported().pure().gap('positive.3', 'Positive[3]', { expected: 'True' }).done(),
    feature('VectorQ', 'predicates').unsupported().pure().gap('vectorq.12', 'VectorQ[{1, 2}]', { expected: 'True' }).done(),
    feature('MatrixQ', 'predicates').unsupported().pure().gap('matrixq.row', 'MatrixQ[{{1, 2}}]', { expected: 'True' }).done(),
    feature('ListQ', 'predicates').unsupported().pure().gap('listq.1', 'ListQ[{1}]', { expected: 'True' }).done(),
    feature('StringQ', 'predicates').unsupported().pure().gap('stringq.a', 'StringQ["a"]', { expected: 'True' }).done(),
    feature('TrueQ', 'predicates')
        .unsupported('SILENT WRONG: TrueQ[True]→TrueQ[]; TrueQ[1==1]→TrueQ[1]')
        .pure()
        .gap('trueq.equal', 'TrueQ[1 == 1]', { expected: 'True', notes: 'currently TrueQ[1]' })
        .done(),
    feature('BooleanQ', 'predicates')
        .unsupported('SILENT WRONG: BooleanQ[True] → BooleanQ[]')
        .pure()
        .gap('booleanq.true', 'BooleanQ[True]', { expected: 'True', notes: 'currently BooleanQ[]' })
        .done(),
    feature('Element', 'predicates').unsupported().pure().gap('element.int', 'Element[1, Integers]', { expected: 'True' }).done(),
    feature('SymmetricMatrixQ', 'predicates')
        .unsupported()
        .pure()
        .gap('symmatq.yes', 'SymmetricMatrixQ[{{1, 2}, {2, 1}}]', { expected: 'True' })
        .done(),
];
