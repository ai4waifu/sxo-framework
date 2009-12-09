import { feature } from '@sxo/harness';

export const geometryFeatures = [
    feature('EuclideanDistance', 'geometry')
        .unsupported()
        .pure()
        .gap('euclid.345', 'EuclideanDistance[{0, 0}, {3, 4}]', { expected: '5' })
        .done(),
    feature('Area', 'geometry').unsupported().pure().gap('area.disk', 'Area[Disk[]]', { expected: 'Pi' }).done(),
];
