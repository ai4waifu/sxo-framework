import { feature } from '@sxo/harness';

export const functionFeatures = [
    feature('function_handle', 'function')
        .partial('anonymous `@(x)…` binds and calls via Part→Function ApplyHead; named `@sin` / feval still open')
        .pure()
        .eval('fh.basic', 'f=@(x)x^2; f(4)', '16')
        .gap('fh.feval', 'feval(@sin, 0)', { expected: '0', notes: 'FunctionHandle[Sin] not yet applied by feval' })
        .done(),
];
