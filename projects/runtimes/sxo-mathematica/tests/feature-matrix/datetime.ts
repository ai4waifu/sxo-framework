import { feature } from '@sxo/harness';

export const datetimeFeatures = [
    feature('DateObject', 'datetime').unsupported().pure().gap('dateobject.ymd', 'DateObject[{2020, 1, 1}]', { expected: '...' }).done(),
    feature('DatePlus', 'datetime').unsupported().pure().gap('dateplus.today', 'DatePlus[Today, 1]', { expected: '...' }).done(),
];
