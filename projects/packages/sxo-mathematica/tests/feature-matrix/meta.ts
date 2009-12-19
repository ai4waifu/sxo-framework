import { feature } from '@sxo/harness';

export const metaFeatures = [
    feature('Head', 'meta')
        .unsupported('Head evaluates args first: Head[1+2] → Head[3]; Head[{1,2}] unevaluated')
        .pure()
        .gap('head.list', 'Head[{1, 2}]', { expected: 'List' })
        .gap('head.plus', 'Head[a + b]', { expected: 'Plus' })
        .done(),
    feature('Timing', 'meta')
        .unsupported('SILENT WRONG: Timing[1+1] → Timing[2] (arg evaluated, no timing pair)')
        .effectful()
        .gap('timing.plus', 'Timing[1 + 1]', { expected: '{0., 2}' })
        .done(),
    feature('Quiet', 'meta').unsupported().pure().gap('quiet.div0', 'Quiet[1/0]', { expected: 'ComplexInfinity' }).done(),
    feature('Trace', 'meta')
        .unsupported('SILENT WRONG: Trace[1+1] → Trace[2]')
        .pure()
        .gap('trace.plus', 'Trace[1 + 1]', { expected: '{{1+1,2}}' })
        .done(),
    feature('Assert', 'meta')
        .unsupported('SILENT WRONG: Assert[True] → Assert[] (True stripped)')
        .pure()
        .gap('assert.true', 'Assert[True]', { expected: 'Null', notes: 'currently Assert[]' })
        .done(),
    feature('MessageName', 'meta')
        .unsupported('SILENT WRONG: Message[f::x] → Message[f, x] (:: MessageName broken)')
        .pure()
        .gap('message.colon', 'Message[f::x]', { expected: 'Null', notes: 'currently Message[f, x]' })
        .done(),
    feature('Information', 'meta')
        .unsupported('SILENT WRONG: ??Plus / ?Plus → Plus')
        .pure()
        .gap('info.qq', '??Plus', { expected: '...', notes: 'currently returns Plus' })
        .done(),
    feature('MemoryInUse', 'meta').unsupported().effectful().gap('memoryinuse.basic', 'MemoryInUse[]', { expected: '...' }).done(),
    feature('DollarVersion', 'meta').unsupported().pure().gap('version.atom', '$Version', { expected: '...' }).done(),
];
