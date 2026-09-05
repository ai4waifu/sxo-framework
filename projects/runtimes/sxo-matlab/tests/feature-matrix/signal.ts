import { feature } from '@sxo/harness';

export const signalFeatures = [
    feature('fft', 'signal').unsupported().pure().gap('fft.4', 'fft([1, 2, 3, 4])', { expected: '...' }).done(),
    feature('conv', 'signal').unsupported().pure().gap('conv.basic', 'conv([1, 1], [1, -1])', { expected: '[1, 0, -1]' }).done(),
    feature('ifft', 'signal').unsupported().pure().gap('ifft.roundtrip', 'ifft(fft([1, 2, 3, 4]))', { expected: '[1, 2, 3, 4]' }).done(),
    feature('filter', 'signal').unsupported().pure().gap('filter.basic', 'filter([1], [1, -0.5], ones(1, 5))', { expected: '...' }).done(),
    feature('conv2', 'signal').unsupported().pure().gap('conv2.basic', 'conv2([1, 2; 3, 4], [1, 1; 1, 1])', { expected: '...' }).done(),
];
