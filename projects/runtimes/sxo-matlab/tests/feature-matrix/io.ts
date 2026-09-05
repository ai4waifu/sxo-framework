import { feature } from '@sxo/harness';

export const ioFeatures = [
    feature('disp', 'io').unsupported().effectful().gap('disp.1', 'disp(1)', { expected: '1' }).done(),
    feature('jsonencode', 'io').unsupported().pure().gap('jsonencode.struct', "jsonencode(struct('a', 1))", { expected: '{"a":1}' }).done(),
    feature('readmatrix', 'io')
        .unsupported()
        .effectful()
        .gap('readmatrix.csv', "readmatrix('x.csv')", { expected: 'UnsupportedOperation' })
        .done(),
    feature('jsondecode', 'io').unsupported().pure().gap('jsondecode.obj', 'jsondecode(\'{"a":1}\')', { expected: '...' }).done(),
    feature('pwd', 'io').unsupported().effectful().gap('pwd.basic', 'pwd', { expected: '...' }).done(),
    feature('fullfile', 'io').unsupported().pure().gap('fullfile.ab', "fullfile('a', 'b')", { expected: '...' }).done(),
];
