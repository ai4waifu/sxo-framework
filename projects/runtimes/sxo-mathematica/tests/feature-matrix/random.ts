import { feature } from '@sxo/harness';

export const randomFeatures = [
    feature('RandomInteger', 'random').unsupported().effectful().gap('randominteger.10', 'RandomInteger[10]', { expected: '...' }).done(),
    feature('SeedRandom', 'random')
        .unsupported('SeedRandom[1]; RandomInteger[5] → RandomInteger[5] (no seed / no draw)')
        .stateful()
        .gap('seedrandom.then', 'SeedRandom[1]; RandomInteger[5]', { expected: '...' })
        .done(),
];
