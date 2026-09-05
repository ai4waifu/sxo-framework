import { matrix } from '@sxo/harness';
import { arithmeticFeatures } from './arithmetic.js';
import { arrayFeatures } from './array.js';
import { bitwiseFeatures } from './bitwise.js';
import { calculusFeatures } from './calculus.js';
import { comparisonFeatures } from './comparison.js';
import { constantFeatures } from './constant.js';
import { controlFeatures } from './control.js';
import { datetimeFeatures } from './datetime.js';
import { elementaryFeatures } from './elementary.js';
import { fitFeatures } from './fit.js';
import { functionFeatures } from './function.js';
import { functionalFeatures } from './functional.js';
import { indexingFeatures } from './indexing.js';
import { interopFeatures } from './interop.js';
import { ioFeatures } from './io.js';
import { linearAlgebraFeatures } from './linear_algebra.js';
import { logicFeatures } from './logic.js';
import { matrixFeatures } from './matrix.js';
import { metaFeatures } from './meta.js';
import { numericFeatures } from './numeric.js';
import { odeFeatures } from './ode.js';
import { oopFeatures } from './oop.js';
import { parallelFeatures } from './parallel.js';
import { plotFeatures } from './plot.js';
import { polynomialFeatures } from './polynomial.js';
import { predicatesFeatures } from './predicates.js';
import { randomFeatures } from './random.js';
import { sessionFeatures } from './session.js';
import { signalFeatures } from './signal.js';
import { simplifyFeatures } from './simplify.js';
import { solveFeatures } from './solve.js';
import { statsFeatures } from './stats.js';
import { stringFeatures } from './string.js';
import { symbolicFeatures } from './symbolic.js';
import { tableFeatures } from './table.js';
import { typesFeatures } from './types.js';
import { uiFeatures } from './ui.js';

/** matlab dialect capability matrix (tests-only truth source). */
export const featureMatrix = matrix(
    ...arithmeticFeatures,
    ...arrayFeatures,
    ...bitwiseFeatures,
    ...calculusFeatures,
    ...comparisonFeatures,
    ...constantFeatures,
    ...controlFeatures,
    ...datetimeFeatures,
    ...elementaryFeatures,
    ...fitFeatures,
    ...functionFeatures,
    ...functionalFeatures,
    ...indexingFeatures,
    ...interopFeatures,
    ...ioFeatures,
    ...linearAlgebraFeatures,
    ...logicFeatures,
    ...matrixFeatures,
    ...metaFeatures,
    ...numericFeatures,
    ...odeFeatures,
    ...oopFeatures,
    ...parallelFeatures,
    ...plotFeatures,
    ...polynomialFeatures,
    ...predicatesFeatures,
    ...randomFeatures,
    ...sessionFeatures,
    ...signalFeatures,
    ...simplifyFeatures,
    ...solveFeatures,
    ...statsFeatures,
    ...stringFeatures,
    ...symbolicFeatures,
    ...tableFeatures,
    ...typesFeatures,
    ...uiFeatures,
);
