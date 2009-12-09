import { feature } from '@sxo/harness';

export const functionFeatures = [
    feature('function_handle', 'function')
        .unsupported('oak error on @(x); feval(@sin,0) strips @')
        .pure()
        .gap('fh.basic', 'f=@(x)x^2; f(4)', { expected: '16' })
        .gap('fh.feval', 'feval(@sin, 0)', { expected: '0', notes: 'currently feval(sin, 0)' })
        .done(),
];
