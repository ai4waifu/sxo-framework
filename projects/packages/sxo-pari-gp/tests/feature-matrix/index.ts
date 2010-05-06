import { matrix } from '@sxo/harness';
import { arithmeticFeatures } from './arithmetic.js';
import { modularFeatures } from './modular.js';
import { numberTheoryFeatures } from './number_theory.js';
import { polynomialFeatures } from './polynomial.js';
import { sessionFeatures } from './session.js';

/** pari-gp dialect capability matrix (tests-only truth source). */
export const featureMatrix = matrix(
    ...arithmeticFeatures,
    ...numberTheoryFeatures,
    ...modularFeatures,
    ...polynomialFeatures,
    ...sessionFeatures,
);
