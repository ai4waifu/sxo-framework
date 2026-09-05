export type {
    CaseKind,
    FeatureBackend,
    FeatureCase,
    FeatureEffect,
    FeatureEntry,
    FeatureHost,
    FeatureMatrix,
    FeatureStatus,
    MatrixValidationIssue,
    MatrixValidationResult,
} from './matrix/index.js';
export { assertValidFeatureMatrix, listRunnableCases, validateFeatureMatrix } from './matrix/index.js';
export type { FeatureGapRow } from './reporters/index.js';
export { listGaps, summarizeMatrix, toConsoleRows, toMarkdownTable } from './reporters/index.js';
