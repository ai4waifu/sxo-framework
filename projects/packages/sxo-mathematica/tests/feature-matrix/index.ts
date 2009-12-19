import { matrix } from '@sxo/harness';
import { algebraFeatures } from './algebra.js';
import { arithmeticFeatures } from './arithmetic.js';
import { associationFeatures } from './association.js';
import { attributesFeatures } from './attributes.js';
import { bitwiseFeatures } from './bitwise.js';
import { calculusFeatures } from './calculus.js';
import { comparisonFeatures } from './comparison.js';
import { complexFeatures } from './complex.js';
import { constantFeatures } from './constant.js';
import { controlFeatures } from './control.js';
import { datetimeFeatures } from './datetime.js';
import { elementaryFeatures } from './elementary.js';
import { fitFeatures } from './fit.js';
import { formsFeatures } from './forms.js';
import { functionFeatures } from './function.js';
import { functionalFeatures } from './functional.js';
import { geoFeatures } from './geo.js';
import { geometryFeatures } from './geometry.js';
import { graphFeatures } from './graph.js';
import { graphicsFeatures } from './graphics.js';
import { holdFeatures } from './hold.js';
import { ioFeatures } from './io.js';
import { knowledgeFeatures } from './knowledge.js';
import { linearAlgebraFeatures } from './linear_algebra.js';
import { listFeatures } from './list.js';
import { logicFeatures } from './logic.js';
import { metaFeatures } from './meta.js';
import { numberTheoryFeatures } from './number_theory.js';
import { numericFeatures } from './numeric.js';
import { parallelFeatures } from './parallel.js';
import { parseFeatures } from './parse.js';
import { patternFeatures } from './pattern.js';
import { plotFeatures } from './plot.js';
import { predicatesFeatures } from './predicates.js';
import { randomFeatures } from './random.js';
import { ruleFeatures } from './rule.js';
import { sessionFeatures } from './session.js';
import { simplifyFeatures } from './simplify.js';
import { solveFeatures } from './solve.js';
import { specialFeatures } from './special.js';
import { statsFeatures } from './stats.js';
import { stringFeatures } from './string.js';
import { unitsFeatures } from './units.js';

/** mathematica dialect capability matrix (tests-only truth source). */
export const featureMatrix = matrix(
    ...algebraFeatures,
    ...arithmeticFeatures,
    ...associationFeatures,
    ...attributesFeatures,
    ...bitwiseFeatures,
    ...calculusFeatures,
    ...comparisonFeatures,
    ...complexFeatures,
    ...constantFeatures,
    ...controlFeatures,
    ...datetimeFeatures,
    ...elementaryFeatures,
    ...fitFeatures,
    ...formsFeatures,
    ...functionFeatures,
    ...functionalFeatures,
    ...geoFeatures,
    ...geometryFeatures,
    ...graphFeatures,
    ...graphicsFeatures,
    ...holdFeatures,
    ...ioFeatures,
    ...knowledgeFeatures,
    ...linearAlgebraFeatures,
    ...listFeatures,
    ...logicFeatures,
    ...metaFeatures,
    ...numberTheoryFeatures,
    ...numericFeatures,
    ...parallelFeatures,
    ...parseFeatures,
    ...patternFeatures,
    ...plotFeatures,
    ...predicatesFeatures,
    ...randomFeatures,
    ...ruleFeatures,
    ...sessionFeatures,
    ...simplifyFeatures,
    ...solveFeatures,
    ...specialFeatures,
    ...statsFeatures,
    ...stringFeatures,
    ...unitsFeatures,
);
