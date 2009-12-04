export type {
    CaseKind,
    FeatureBackend,
    FeatureCase,
    FeatureCaseRunFail,
    FeatureCaseRunGap,
    FeatureCaseRunOk,
    FeatureCaseRunResult,
    FeatureEffect,
    FeatureEntry,
    FeatureFixtureHooks,
    FeatureHost,
    FeatureMatrix,
    FeatureStatus,
    MatrixValidationIssue,
    MatrixValidationResult,
} from './matrix/index.js';
export {
    FeatureEntryBuilder,
    assertValidFeatureMatrix,
    entry,
    evalCase,
    feature,
    gapCase,
    listRunnableCases,
    matrix,
    negativeCase,
    parseCase,
    plotCase,
    roundtripCase,
    runFeatureCase,
    validateFeatureMatrix,
} from './matrix/index.js';
export type {
    FeatureCaseOptions,
    GapCaseOptions,
    NegativeCaseOptions,
    PlotCaseOptions,
} from './matrix/index.js';
export type { FeatureGapRow } from './reporters/index.js';
export { listGaps, summarizeMatrix, toConsoleRows, toMarkdownTable } from './reporters/index.js';
export { loadDialectFeatureMatrix, reportDialectFeatures } from './report-features.js';
