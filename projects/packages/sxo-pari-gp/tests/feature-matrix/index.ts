import { matrix } from '@sxo/harness';
import { arithmeticFeatures } from './arithmetic.js';

export const featureMatrix = matrix(...arithmeticFeatures);
