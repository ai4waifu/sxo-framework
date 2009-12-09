import { feature } from '@sxo/harness';

export const stringFeatures = [
    feature('StringLength', 'string').unsupported().pure().gap('strlen.ab', 'StringLength["ab"]', { expected: '2' }).done(),
    feature('ToString', 'string')
        .unsupported('args evaluated first: ToString[1+1] → ToString[2]')
        .pure()
        .gap('tostring.plus', 'ToString[1 + 1]', { expected: '"2"' })
        .done(),
    feature('ToExpression', 'string').unsupported().pure().gap('toexpression.plus', 'ToExpression["1+1"]', { expected: '2' }).done(),
    feature('StringTake', 'string').unsupported().pure().gap('stringtake.2', 'StringTake["abcd", 2]', { expected: '"ab"' }).done(),
    feature('StringReverse', 'string').unsupported().pure().gap('stringreverse.ab', 'StringReverse["ab"]', { expected: '"ba"' }).done(),
    feature('StringJoin', 'string').unsupported().pure().gap('stringjoin.ab', 'StringJoin["a", "b"]', { expected: '"ab"' }).done(),
    feature('Characters', 'string').unsupported().pure().gap('characters.ab', 'Characters["ab"]', { expected: '{"a", "b"}' }).done(),
    feature('StringTrim', 'string').unsupported().pure().gap('stringtrim.a', 'StringTrim[" a "]', { expected: '"a"' }).done(),
    feature('Hash', 'string').unsupported().pure().gap('hash.a', 'Hash["a"]', { expected: '...' }).done(),
    feature('HammingDistance', 'string')
        .unsupported()
        .pure()
        .gap('hamming.bits', 'HammingDistance[{1, 0, 1}, {1, 1, 0}]', { expected: '2' })
        .done(),
    feature('EditDistance', 'string')
        .unsupported()
        .pure()
        .gap('editdistance.kitten', 'EditDistance["kitten", "sitting"]', { expected: '3' })
        .done(),
];
