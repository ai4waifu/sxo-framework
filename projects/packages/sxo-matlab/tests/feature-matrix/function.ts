import { feature } from '@sxo/harness';

export const functionFeatures = [
    feature('function_handle', 'function')
        .supported()
        .pure()
        .notes('anonymous `@(x)…` via Function+Part; `feval(@sin,0)` unwraps FunctionHandle')
        .eval('fh.basic', 'f=@(x)x^2; f(4)', '16')
        .eval('fh.feval', 'feval(@sin, 0)', '0')
        .done(),
];
