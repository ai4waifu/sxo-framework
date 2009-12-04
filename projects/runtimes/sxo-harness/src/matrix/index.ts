export type {
    CaseKind,
    FeatureBackend,
    FeatureCase,
    FeatureEffect,
    FeatureEntry,
    FeatureHost,
    FeatureMatrix,
    FeatureStatus,
} from './types.js';
export type {
    FeatureCaseOptions,
    GapCaseOptions,
    NegativeCaseOptions,
    PlotCaseOptions,
} from './builders.js';
export {
    FeatureEntryBuilder,
    entry,
    evalCase,
    feature,
    gapCase,
    matrix,
    negativeCase,
    parseCase,
    plotCase,
    roundtripCase,
} from './builders.js';
export type { MatrixValidationIssue, MatrixValidationResult } from './validate.js';
export { assertValidFeatureMatrix, listRunnableCases, validateFeatureMatrix } from './validate.js';
export type {
    FeatureCaseRunFail,
    FeatureCaseRunGap,
    FeatureCaseRunOk,
    FeatureCaseRunResult,
    FeatureFixtureHooks,
} from './runner.js';
export { runFeatureCase } from './runner.js';
