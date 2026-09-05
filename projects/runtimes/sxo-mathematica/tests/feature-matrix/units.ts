import { feature } from '@sxo/harness';

export const unitsFeatures = [
    feature('Quantity', 'units').unsupported().pure().gap('quantity.m', 'Quantity[1, "Meters"]', { expected: 'Quantity[1, "Meters"]' }).done(),
    feature('UnitConvert', 'units')
        .unsupported()
        .pure()
        .gap('unitconvert.m_cm', 'UnitConvert[Quantity[1, "m"], "cm"]', { expected: 'Quantity[100, "cm"]' })
        .done(),
];
