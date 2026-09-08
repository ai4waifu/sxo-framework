import { feature } from '@sxo/harness';

export const constantFeatures = [
    feature('pi', 'constant').partial().pure().eval('pi.symbol', 'pi', 'pi').done(),
    feature('true', 'constant').partial().pure().eval('true.atom', 'true', 'true').done(),
    feature('false', 'constant').partial().pure().eval('false.atom', 'false', 'false').done(),
    feature('nan', 'constant')
        .partial('lowercase `nan` lowers to Indeterminate and renders capitalized `NaN`')
        .pure()
        .eval('nan.lower', 'nan', 'NaN')
        .done(),
    feature('eps', 'constant').unsupported().pure().gap('eps.atom', 'eps', { expected: '...' }).done(),
    feature('inf', 'constant')
        .partial('lowercase `inf` lowers to Infinity and renders capitalized `Inf`')
        .pure()
        .eval('inf.atom', 'inf', 'Inf')
        .done(),
    feature('NaN', 'constant').partial().pure().eval('NaN.capital', 'NaN', 'NaN').done(),
    feature('flintmax', 'constant').unsupported().pure().gap('flintmax.atom', 'flintmax', { expected: '...' }).done(),
];
