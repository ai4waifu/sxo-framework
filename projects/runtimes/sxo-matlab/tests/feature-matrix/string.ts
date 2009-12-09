import { feature } from '@sxo/harness';

export const stringFeatures = [
    feature('num2str', 'string').unsupported().pure().gap('num2str.3', 'num2str(3)', { expected: "'3'" }).done(),
    feature('str2num', 'string').unsupported().pure().gap('str2num.3', "str2num('3')", { expected: '3' }).done(),
    feature('contains', 'string').unsupported().pure().gap('contains.b', "contains('abc', 'b')", { expected: '1' }).done(),
    feature('string', 'string').unsupported().pure().gap('string.ctor', "string('ab')", { expected: '"ab"' }).done(),
    feature('sprintf', 'string').unsupported().pure().gap('sprintf.d', "sprintf('%d', 1)", { expected: "'1'" }).done(),
    feature('strcat', 'string').unsupported().pure().gap('strcat.ab', "strcat('a', 'b')", { expected: "'ab'" }).done(),
    feature('strcmp', 'string').unsupported().pure().gap('strcmp.eq', "strcmp('a', 'a')", { expected: '1' }).done(),
    feature('strjoin', 'string')
        .unsupported("SILENT WRONG: cell brace stripped — strjoin({'a','b'},',') → strjoin('a', 'b', ',')")
        .pure()
        .gap('strjoin.ab', "strjoin({'a', 'b'}, ',')", { expected: "'a,b'" })
        .done(),
    feature('char', 'string').unsupported().pure().gap('char.65', 'char(65)', { expected: "'A'" }).done(),
    feature('string_plus', 'string')
        .unsupported()
        .pure()
        .gap('string.plus', '"hello" + "world"', { expected: '"helloworld"' })
        .gap('string.append', 'append("a", "b")', { expected: '"ab"' })
        .done(),
    feature('startsWith', 'string').unsupported().pure().gap('startswith.a', 'startsWith("abc", "a")', { expected: '1' }).done(),
    feature('endsWith', 'string').unsupported().pure().gap('endswith.c', 'endsWith("abc", "c")', { expected: '1' }).done(),
    feature('erase', 'string').unsupported().pure().gap('erase.b', 'erase("abc", "b")', { expected: '"ac"' }).done(),
    feature('extractAfter', 'string').unsupported().pure().gap('extractafter.a', 'extractAfter("abc", "a")', { expected: '"bc"' }).done(),
    feature('matches', 'string').unsupported().pure().gap('matches.digits', 'matches("abc", digitsPattern)', { expected: '0' }).done(),
    feature('wildcardPattern', 'string').unsupported().pure().gap('wildcard.m', 'wildcardPattern("*.m")', { expected: '...' }).done(),
];
