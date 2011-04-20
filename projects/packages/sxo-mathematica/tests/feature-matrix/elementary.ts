import { feature } from '@sxo/harness';

export const elementaryFeatures = [
    feature('Sin', 'elementary').partial('exact `Sin` on tested integer arguments only').pure().eval('sin.0', 'Sin[0]', '0').done(),
    feature('Cos', 'elementary').partial('exact `Cos` on tested integer arguments only').pure().eval('cos.0', 'Cos[0]', '1').done(),
    feature('Tan', 'elementary').partial('exact `Tan` on tested integer arguments only').pure().eval('tan.0', 'Tan[0]', '0').done(),
    feature('Exp', 'elementary').partial('exact `Exp` on tested integer arguments only').pure().eval('exp.0', 'Exp[0]', '1').done(),
    feature('Log', 'elementary').partial('exact `Log` on tested integer arguments only').pure().eval('log.1', 'Log[1]', '0').done(),
    feature('ArcSin', 'elementary').partial('exact `ArcSin[0]` fold').pure().eval('arcsin.0', 'ArcSin[0]', '0').done(),
    feature('Sinh', 'elementary').partial('exact `Sinh[0]` fold').pure().eval('sinh.0', 'Sinh[0]', '0').done(),
    feature('Cosh', 'elementary').partial('exact `Cosh[0]` fold').pure().eval('cosh.0', 'Cosh[0]', '1').done(),
    feature('LogE', 'elementary').partial('exact `Log[E]` fold').pure().eval('log.e', 'Log[E]', '1').done(),
    feature('ArcTan', 'elementary').partial('exact `ArcTan[1]` fold to `Pi/4`').pure().eval('arctan.1', 'ArcTan[1]', 'Pi/4').done(),
];
