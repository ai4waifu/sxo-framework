import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('Sin', 'elementary').partial('exact `Sin` on tested integer arguments only').pure().eval('sin.0', 'Sin[0]', '0').done(),
    feature('Cos', 'elementary').partial('exact `Cos` on tested integer arguments only').pure().eval('cos.0', 'Cos[0]', '1').done(),
    feature('Tan', 'elementary').partial('exact `Tan` on tested integer arguments only').pure().eval('tan.0', 'Tan[0]', '0').done(),
    feature('Exp', 'elementary').partial('exact `Exp` on tested integer arguments only').pure().eval('exp.0', 'Exp[0]', '1').done(),
    feature('Log', 'elementary').partial('exact `Log` on tested integer arguments only').pure().eval('log.1', 'Log[1]', '0').done(),
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
