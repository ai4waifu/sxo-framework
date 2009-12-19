import { feature } from '@sxo/harness';

export const graphicsFeatures = [
    feature('RGBColor', 'graphics').unsupported().pure().gap('rgb.red', 'RGBColor[1, 0, 0]', { expected: 'RGBColor[1, 0, 0]' }).done(),
    feature('Style', 'graphics').unsupported().pure().gap('style.bold', 'Style[x, Bold]', { expected: 'Style[x, Bold]' }).done(),
];
