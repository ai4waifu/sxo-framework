import { feature } from '@sxo/harness';

export const attributesFeatures = [
    feature('HoldAll', 'attributes').planned().unevaluated().gap('attr.holdall', 'Attributes[Hold]', { expected: '{HoldAll}' }).done(),
    feature('Listable', 'attributes').planned().pure().gap('attr.listable', 'Sin[{0, Pi}]', { expected: '{0, 0}' }).done(),
    feature('Orderless', 'attributes').planned().pure().gap('attr.orderless', 'Plus[b, a]', { expected: 'a + b' }).done(),
    feature('AttributesPlus', 'attributes')
        .planned('Attributes[Plus] unevaluated (Orderless/Flat/Listable contract)')
        .pure()
        .gap('attr.plus', 'Attributes[Plus]', { expected: '{Flat, Listable, NumericFunction, OneIdentity, Orderless, Protected}' })
        .done(),
];
