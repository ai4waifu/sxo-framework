import { feature } from '@sxo/harness';

export const metaFeatures = [
    feature('Head', 'meta')
        .unsupported('Head evaluates args first: Head[1+2] → Head[3]; Head[{1,2}] unevaluated')
        .pure()
        .gap('head.list', 'Head[{1, 2}]', { expected: 'List' })
        .gap('head.plus', 'Head[a + b]', { expected: 'Plus' })
        .done(),
    feature('Timing', 'meta')
        .unsupported('HoldAll Form kept; no wall-clock Timing pair runtime')
        .effectful()
        .gap('timing.plus', 'Timing[1 + 1]', {
            expected: '{0., 2}',
            notes: 'must stay Timing[1 + 1], not Timing[2]; pair {time, value} still unsupported',
        })
        .done(),
    feature('Quiet', 'meta').unsupported().pure().gap('quiet.div0', 'Quiet[1/0]', { expected: 'ComplexInfinity' }).done(),
    feature('Trace', 'meta')
        .unsupported('HoldAll Form kept; no Trace step list runtime')
        .pure()
        .gap('trace.plus', 'Trace[1 + 1]', {
            expected: '{{1+1,2}}',
            notes: 'must stay Trace[1 + 1], not Trace[2]; step-list Trace still unsupported',
        })
        .done(),
    feature('Assert', 'meta')
        .supported()
        .pure()
        .notes('Assert lowers to Branch → Null / Reject')
        .eval('assert.true', 'Assert[True]', 'Null')
        .eval('assert.equal', 'Assert[1 == 1]', 'Null')
        .done(),
    feature('MessageName', 'meta')
        .unsupported('MessageName Form kept via ::; Message runtime not implemented')
        .pure()
        .gap('message.colon', 'Message[f::x]', {
            expected: 'Null',
            notes: 'parses as Message[MessageName[f, x]] / renders Message[f::x]; not Message[f, x]',
        })
        .done(),
    feature('Information', 'meta')
        .unsupported('Information Form kept via ??; no Information runtime')
        .pure()
        .gap('info.qq', '??Plus', {
            expected: '...',
            notes: 'parses as Information[Plus]; not silent bare Plus / error node',
        })
        .done(),
    feature('MemoryInUse', 'meta').unsupported().effectful().gap('memoryinuse.basic', 'MemoryInUse[]', { expected: '...' }).done(),
    feature('DollarVersion', 'meta').unsupported().pure().gap('version.atom', '$Version', { expected: '...' }).done(),
];
