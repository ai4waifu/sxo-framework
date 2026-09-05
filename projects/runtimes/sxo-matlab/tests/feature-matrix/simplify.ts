import { feature } from '@sxo/harness';

export const simplifyFeatures = [feature('simplify', 'simplify').supported().pure().eval('simplify.trig', 'sin(x)^2 + cos(x)^2', '1').done()];
