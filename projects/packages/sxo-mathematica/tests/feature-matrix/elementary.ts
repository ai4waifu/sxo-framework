import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('Sin', 'elementary').supported().pure().eval('sin.0', 'Sin[0]', '0').done(),
    feature('Cos', 'elementary').supported().pure().eval('cos.0', 'Cos[0]', '1').done(),
    feature('Tan', 'elementary').supported().pure().eval('tan.0', 'Tan[0]', '0').done(),
    feature('Exp', 'elementary').supported().pure().eval('exp.0', 'Exp[0]', '1').done(),
    feature('Log', 'elementary').supported().pure().eval('log.1', 'Log[1]', '0').done(),
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
