import { feature } from '@sxo/harness';

export const constantFeatures = [
    feature('pi', 'constant').partial().pure().eval('pi.symbol', 'pi', 'pi').done(),
    feature('true', 'constant').partial().pure().eval('true.atom', 'true', 'true').done(),
    feature('false', 'constant').partial().pure().eval('false.atom', 'false', 'false').done(),
    feature('nan', 'constant')
        .partial('lowercase atom retained; NaN arithmetic / isnan contract incomplete')
        .pure()
        .eval('nan.lower', 'nan', 'nan')
        .done(),
    feature('eps', 'constant').unsupported().pure().gap('eps.atom', 'eps', { expected: '...' }).done(),
    feature('inf', 'constant').partial('symbol retained; no Inf arithmetic contract yet').pure().eval('inf.atom', 'inf', 'inf').done(),
    feature('NaN', 'constant').partial().pure().eval('NaN.capital', 'NaN', 'NaN').done(),
    feature('flintmax', 'constant').unsupported().pure().gap('flintmax.atom', 'flintmax', { expected: '...' }).done(),
];
