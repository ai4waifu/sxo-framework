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
    assertValidFeatureMatrix,
    listRunnableCases,
    runFeatureCase,
    validateFeatureMatrix,
} from './matrix/index.js';
export type { FeatureGapRow } from './reporters/index.js';
export { listGaps, summarizeMatrix, toConsoleRows, toMarkdownTable } from './reporters/index.js';
export { loadDialectFeatureMatrix, reportDialectFeatures } from './report-features.js';
