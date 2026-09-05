import { feature } from '@sxo/harness';

export const oopFeatures = [
    feature('classdef', 'oop')
        .unsupported('SILENT WRONG: classdef Foo, end → Foo')
        .stateful()
        .gap('classdef.foo', 'classdef Foo, end', { expected: '...', notes: 'currently returns Foo' })
        .done(),
];
