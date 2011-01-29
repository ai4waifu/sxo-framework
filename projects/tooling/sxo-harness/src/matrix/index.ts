export type {
    FeatureCaseOptions,
    GapCaseOptions,
    NegativeCaseOptions,
    PlotCaseOptions,
    SuboptimalCaseOptions,
    WrongCaseOptions,
} from './builders.js';
export {
    cosmeticCase,
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
    suboptimalCase,
    wrongCase,
} from './builders.js';
export type { WellKnownFeatureCaseFlag } from './flags.js';
export { FEATURE_CASE_FLAGS, mergeCaseFlags } from './flags.js';
export type {
    FeatureCaseRunFail,
    FeatureCaseRunGap,
    FeatureCaseRunOk,
    FeatureCaseRunOptions,
    FeatureCaseRunResult,
    FeatureCaseRunWrong,
    FeatureFixtureHooks,
    IsolatedEvalResult,
    IsolatedEvalSpec,
} from './runner.js';
export { runFeatureCase, runIsolatedEval } from './runner.js';
export type {
    CaseKind,
    FeatureBackend,
    FeatureBinaryIdentity,
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
