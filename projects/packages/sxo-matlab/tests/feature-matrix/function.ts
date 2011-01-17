import { feature } from '@sxo/harness';

export const functionFeatures = [
    feature('function_handle', 'function')
        .partial('anonymous `@(x)…` and `feval(@sin,0)` on tested forms only')
        .pure()
        .eval('fh.basic', 'f=@(x)x^2; f(4)', '16')
        .eval('fh.feval', 'feval(@sin, 0)', '0')
        .done(),
];
