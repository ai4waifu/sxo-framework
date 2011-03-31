import { feature } from '@sxo/harness';

export const predicatesFeatures = [
    feature('PossibleZeroQ', 'predicates').partial('numeric zero test only').pure().eval('possiblezeroq.0', 'PossibleZeroQ[0]', 'True').done(),
    feature('NumericQ', 'predicates').partial('numeric atom test only').pure().eval('numericq.1', 'NumericQ[1]', 'True').done(),
    feature('IntegerQ', 'predicates')
        .partial('exact integer and integer-valued rationals only')
        .pure()
        .eval('integerq.1', 'IntegerQ[1]', 'True')
        .done(),
    feature('AtomQ', 'predicates').partial('non-compound terms only').pure().eval('atomq.1', 'AtomQ[1]', 'True').done(),
    feature('NumberQ', 'predicates').partial('numeric atom test only').pure().eval('numberq.12', 'NumberQ[1.2]', 'True').done(),
    feature('EvenQ', 'predicates')
        .partial('exact integer parity only')
        .pure()
        .eval('evenq.2', 'EvenQ[2]', 'True')
        .eval('evenq.3', 'EvenQ[3]', 'False')
        .done(),
    feature('Positive', 'predicates').partial('numeric atom sign test only').pure().eval('positive.3', 'Positive[3]', 'True').done(),
    feature('VectorQ', 'predicates').partial('flat ordered `Collection` only').pure().eval('vectorq.12', 'VectorQ[{1, 2}]', 'True').done(),
    feature('MatrixQ', 'predicates')
        .partial('rectangular nested row `Collection` only')
        .pure()
        .eval('matrixq.row', 'MatrixQ[{{1, 2}}]', 'True')
        .done(),
    feature('ListQ', 'predicates').partial('ordered `Collection` only').pure().eval('listq.1', 'ListQ[{1}]', 'True').done(),
    feature('StringQ', 'predicates').partial('`Atom::String` only').pure().eval('stringq.a', 'StringQ["a"]', 'True').done(),
    feature('TrueQ', 'predicates')
        .partial('`TrueQ` on tested boolean and equality forms only')
        .pure()
        .eval('trueq.true', 'TrueQ[True]', 'True')
        .eval('trueq.equal', 'TrueQ[1 == 1]', 'True')
        .eval('trueq.false', 'TrueQ[False]', 'False')
        .done(),
    feature('BooleanQ', 'predicates').partial('`Atom::Boolean` only').pure().eval('booleanq.true', 'BooleanQ[True]', 'True').done(),
    feature('Element', 'predicates')
        .partial('`Integers` domain on exact integer atoms only')
        .pure()
        .eval('element.int', 'Element[1, Integers]', 'True')
        .done(),
    feature('SymmetricMatrixQ', 'predicates')
        .partial('typed numeric matrix symmetry predicate only. Surface is `0`/`1`, not `True`/`False`')
        .pure()
        .eval('symmatq.yes', 'SymmetricMatrixQ[{{1, 2}, {2, 1}}]', '1')
        .eval('symmatq.no', 'SymmetricMatrixQ[{{1, 2}, {3, 4}}]', '0')
        .done(),
];
