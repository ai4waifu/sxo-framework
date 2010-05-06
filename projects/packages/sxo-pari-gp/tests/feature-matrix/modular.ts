import { feature } from '@sxo/harness';

/** Modular arithmetic (`Mod` / lift) — planned until Modular / FiniteField domains land. */
export const modularFeatures = [
    feature('Mod', 'modular')
        .planned('Mod(a, n) Form + Athena Modular domain')
        .pure()
        .gap('mod.basic', 'Mod(7, 5)', { expected: 'Mod(2, 5)' })
        .done(),
    feature('lift', 'modular')
        .planned('lift(Mod(...)) to integer')
        .pure()
        .gap('lift.mod', 'lift(Mod(7, 5))', { expected: '2' })
        .done(),
];
