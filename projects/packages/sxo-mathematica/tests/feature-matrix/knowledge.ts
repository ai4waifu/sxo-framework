import { feature } from '@sxo/harness';

export const knowledgeFeatures = [
    feature('Entity', 'knowledge').planned().effectful().gap('entity.country', 'Entity["Country", "Spain"]', { expected: '...' }).done(),
];
