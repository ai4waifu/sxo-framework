import { feature } from '@sxo/harness';

export const associationFeatures = [
    feature('Association', 'association')
        .unsupported('Association[…] unevaluated; <|…|> oak error node')
        .pure()
        .gap('assoc.head', 'Association[a -> 1, b -> 2]', { expected: '<|a -> 1, b -> 2|>' })
        .gap('assoc.literal', '<|a -> 1|>', { expected: '<|a -> 1|>', notes: 'oak error node' })
        .done(),
];
