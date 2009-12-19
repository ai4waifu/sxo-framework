import { feature } from '@sxo/harness';

export const numericFeatures = [
    feature('N', 'numeric')
        .unsupported()
        .pure()
        .gap('n.pi', 'N[Pi]', { expected: '3.14159' })
        .gap('n.prec', 'N[1/3, 20]', { expected: '0.33333333333333333333' })
        .done(),
    feature('ExactInteger', 'numeric').supported().pure().eval('bigint.add', '99999999999999999999 + 1', '100000000000000000000').done(),
    feature('Interval', 'numeric')
        .unsupported()
        .pure()
        .gap('interval.basic', 'Interval[{1, 2}]', { expected: 'Interval[{1, 2}]', notes: 'echo only; no arithmetic' })
        .done(),
    feature('IntervalUnion', 'numeric')
        .unsupported()
        .pure()
        .gap('intervalunion.adj', 'IntervalUnion[Interval[{1, 2}], Interval[{2, 3}]]', { expected: 'Interval[{1, 3}]' })
        .done(),
    feature('IntervalIntersection', 'numeric')
        .unsupported()
        .pure()
        .gap('intervalintersection.overlap', 'IntervalIntersection[Interval[{1, 3}], Interval[{2, 4}]]', { expected: 'Interval[{2, 3}]' })
        .done(),
    feature('Around', 'numeric').unsupported().pure().gap('around.basic', 'Around[1.23, 0.01]', { expected: 'Around[1.23, 0.01]' }).done(),
];
