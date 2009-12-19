import { feature } from '@sxo/harness';

export const constantFeatures = [
    feature('Pi', 'constant').partial('symbol retained; no numeric value without N').pure().eval('pi.symbol', 'Pi', 'Pi').done(),
    feature('I', 'constant').partial('symbol only; Re/Im/Conjugate unevaluated').pure().eval('i.symbol', 'I', 'I').done(),
    feature('True', 'constant').supported().pure().notes('typed Boolean atom').eval('true.atom', 'True', 'True').done(),
    feature('False', 'constant').supported().pure().notes('typed Boolean atom').eval('false.atom', 'False', 'False').done(),
    feature('Null', 'constant').supported().pure().notes('typed Null atom').eval('null.atom', 'Null', 'Null').done(),
    feature('E', 'constant').partial().pure().eval('e.symbol', 'E', 'E').done(),
    feature('Infinity', 'constant')
        .partial('symbol retained; arithmetic with Infinity incomplete')
        .pure()
        .eval('inf.symbol', 'Infinity', 'Infinity')
        .done(),
];
