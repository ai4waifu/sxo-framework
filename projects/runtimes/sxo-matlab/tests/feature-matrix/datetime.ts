import { feature } from '@sxo/harness';

export const datetimeFeatures = [
    feature('datetime_arith', 'datetime')
        .unsupported()
        .pure()
        .gap('datetime.caldays', 'datetime(2020, 1, 1) + caldays(1)', { expected: 'datetime(2020, 1, 2)' })
        .gap('datetime.days', 'datetime(2020, 1, 1) + days(1)', { expected: 'datetime(2020, 1, 2)' })
        .done(),
    feature('caldiff', 'datetime')
        .unsupported()
        .pure()
        .gap('caldiff.year', 'caldiff(datetime(2020, 1, 1), datetime(2021, 1, 1))', { expected: '...' })
        .done(),
];
