import { feature } from '@sxo/harness';

export const simplifyFeatures = [feature('simplify', 'simplify').partial('tested trig identity fold only').pure().eval('simplify.trig', 'sin(x)^2 + cos(x)^2', '1').done()];
