import { feature } from '@sxo/harness';

export const oopFeatures = [
    feature('classdef', 'oop')
        .unsupported('classdef still unsupported (oak Error / Reject); not silent Foo')
        .stateful()
        .gap('classdef.foo', 'classdef Foo, end', {
            expected: '...',
            notes: 'must not evaluate to bare Foo; typed Classdef statement is still a follow-up',
        })
        .done(),
];
