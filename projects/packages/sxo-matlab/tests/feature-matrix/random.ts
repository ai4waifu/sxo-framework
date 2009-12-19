import { feature } from '@sxo/harness';

export const randomFeatures = [
    feature('rand', 'random').unsupported().effectful().gap('rand.2', 'rand(2)', { expected: '...' }).done(),
    feature('randperm', 'random').unsupported().effectful().gap('randperm.5', 'randperm(5)', { expected: '...' }).done(),
];
