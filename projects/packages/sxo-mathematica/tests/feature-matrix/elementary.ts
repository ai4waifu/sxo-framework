import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('Sin', 'elementary').supported().pure().eval('sin.0', 'Sin[0]', '0').done(),
    feature('Cos', 'elementary').supported().pure().eval('cos.0', 'Cos[0]', '1').done(),
    feature('Tan', 'elementary').supported().pure().eval('tan.0', 'Tan[0]', '0').done(),
    feature('Exp', 'elementary')
        .partial('pinned Athena after `fc860cf0` no longer exact-folds `Exp[0]` (removed f64 auto-N without exact specials)')
        .pure()
        .wrong('exp.0', 'Exp[0]', {
            expected: '1',
            flags: ['upstream-athena'],
            notes: 'got unevaluated `Exp[0]` on pin `53a7a5dd`',
        })
        .done(),
    feature('Log', 'elementary')
        .partial('pinned Athena after `fc860cf0` no longer exact-folds `Log[1]`')
        .pure()
        .wrong('log.1', 'Log[1]', {
            expected: '0',
            flags: ['upstream-athena'],
            notes: 'got unevaluated `Log[1]` on pin `53a7a5dd`',
        })
        .done(),
    feature('ArcSin', 'elementary').unsupported().pure().gap('arcsin.0', 'ArcSin[0]', { expected: '0' }).done(),
    feature('Sinh', 'elementary').unsupported().pure().gap('sinh.0', 'Sinh[0]', { expected: '0' }).done(),
    feature('Cosh', 'elementary').unsupported().pure().gap('cosh.0', 'Cosh[0]', { expected: '1' }).done(),
    feature('LogE', 'elementary')
        .unsupported('Log[E] remains unevaluated (canonical Log[E]→1 missing)')
        .pure()
        .gap('log.e', 'Log[E]', { expected: '1' })
        .done(),
    feature('ArcTan', 'elementary').unsupported().pure().gap('arctan.1', 'ArcTan[1]', { expected: 'Pi/4' }).done(),
];
