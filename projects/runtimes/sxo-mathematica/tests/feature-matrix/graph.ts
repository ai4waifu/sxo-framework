import { feature } from '@sxo/harness';

export const graphFeatures = [
    feature('Graph', 'graph').unsupported().pure().gap('graph.path', 'Graph[{1 -> 2, 2 -> 3}]', { expected: '...' }).done(),
    feature('GraphDistance', 'graph')
        .unsupported()
        .pure()
        .gap('graphdistance.path', 'GraphDistance[Graph[{1 -> 2, 2 -> 3}], 1, 3]', { expected: '2' })
        .done(),
];
