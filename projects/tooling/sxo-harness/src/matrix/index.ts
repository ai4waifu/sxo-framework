export type {
    FeatureCaseOptions,
    GapCaseOptions,
    NegativeCaseOptions,
    PlotCaseOptions,
    WrongCaseOptions,
} from './builders.js';
export {
    entry,
    evalCase,
    FeatureEntryBuilder,
    feature,
    gapCase,
    matrix,
    negativeCase,
    parseCase,
    plotCase,
    roundtripCase,
    wrongCase,
} from './builders.js';
export type {
    FeatureCaseRunFail,
    FeatureCaseRunGap,
    FeatureCaseRunOk,
    FeatureCaseRunResult,
    FeatureCaseRunWrong,
    FeatureFixtureHooks,
} from './runner.js';
export { runFeatureCase } from './runner.js';
export type {
    CaseKind,
    FeatureBackend,
    FeatureCase,
    FeatureCaseFlag,
    FeatureEffect,
    FeatureEntry,
    FeatureHost,
    FeatureMatrix,
    FeatureStatus,
} from './types.js';
export type { MatrixValidationIssue, MatrixValidationResult } from './validate.js';
export { assertValidFeatureMatrix, listRunnableCases, validateFeatureMatrix } from './validate.js';
