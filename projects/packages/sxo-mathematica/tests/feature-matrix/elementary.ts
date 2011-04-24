import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('Sin', 'elementary')
        .partial('exact `Sin` on tested integer and Pi arguments')
        .pure()
        .eval('sin.0', 'Sin[0]', '0')
        .eval('sin.pi', 'Sin[Pi]', '0')
        .done(),
    feature('Cos', 'elementary')
        .partial('exact `Cos` on tested integer and Pi arguments')
        .pure()
        .eval('cos.0', 'Cos[0]', '1')
        .eval('cos.pi', 'Cos[Pi]', '-1')
        .done(),
    feature('Tan', 'elementary')
        .partial('exact `Tan` on tested integer and Pi arguments')
        .pure()
        .eval('tan.0', 'Tan[0]', '0')
        .eval('tan.pi', 'Tan[Pi]', '0')
        .done(),
    feature('Exp', 'elementary').partial('exact `Exp` on tested integer arguments only').pure().eval('exp.0', 'Exp[0]', '1').done(),
    feature('Log', 'elementary').partial('exact `Log` on tested integer arguments only').pure().eval('log.1', 'Log[1]', '0').done(),
    feature('ArcSin', 'elementary')
        .partial('exact `ArcSin` on tested boundary integers')
        .pure()
        .eval('arcsin.0', 'ArcSin[0]', '0')
        .eval('arcsin.1', 'ArcSin[1]', 'Pi/2')
        .eval('arcsin.neg1', 'ArcSin[-1]', '-Pi/2')
        .done(),
    feature('Sinh', 'elementary').partial('exact `Sinh[0]` fold').pure().eval('sinh.0', 'Sinh[0]', '0').done(),
    feature('Tanh', 'elementary').partial('exact `Tanh[0]` fold').pure().eval('tanh.0', 'Tanh[0]', '0').done(),
    feature('Cosh', 'elementary').partial('exact `Cosh[0]` fold').pure().eval('cosh.0', 'Cosh[0]', '1').done(),
    feature('LogE', 'elementary').partial('exact `Log[E]` fold').pure().eval('log.e', 'Log[E]', '1').done(),
    feature('ArcCos', 'elementary')
        .partial('exact `ArcCos` on tested boundary integers')
        .pure()
        .eval('arccos.0', 'ArcCos[0]', 'Pi/2')
        .eval('arccos.1', 'ArcCos[1]', '0')
        .eval('arccos.neg1', 'ArcCos[-1]', 'Pi')
        .done(),
    feature('ArcTan', 'elementary')
        .partial('exact `ArcTan` on tested boundary integers')
        .pure()
        .eval('arctan.0', 'ArcTan[0]', '0')
        .eval('arctan.1', 'ArcTan[1]', 'Pi/4')
        .done(),
];
