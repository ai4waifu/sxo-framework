import { feature } from '@sxo/harness';

export const geoFeatures = [
    feature('GeoPosition', 'geo').planned().pure().gap('geoposition.origin', 'GeoPosition[{0, 0}]', { expected: '...' }).done(),
];
