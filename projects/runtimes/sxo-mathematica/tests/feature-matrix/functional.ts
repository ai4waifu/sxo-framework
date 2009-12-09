import { feature } from '@sxo/harness';

export const functionalFeatures = [
    feature('ComposeList', 'functional')
        .unsupported()
        .pure()
        .gap('composelist.fg', 'ComposeList[{f, g}, x]', { expected: '{x, f[x], g[f[x]]}' })
        .done(),
];
