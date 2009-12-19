/** Shared Feature Matrix contracts for dialect packages and harness runners. */

export type FeatureStatus = 'supported' | 'partial' | 'unsupported' | 'planned';

export type FeatureEffect = 'pure' | 'stateful' | 'effectful' | 'unevaluated';

export type CaseKind = 'eval' | 'parse' | 'roundtrip' | 'plot' | 'negative' | 'gap';

/** Execution host for a matrix case or report row. */
export type FeatureHost = 'native' | 'wasm';

/**
 * Backend identity for acceptance runs.
 * External reference backends are local opt-in only.
 */
export type FeatureBackend = 'internal-athena' | 'internal-titan' | 'reference-wolfram' | 'reference-matlab' | 'reference-pari-gp';

export type FeatureCase = {
    id: string;
    kind: CaseKind;
    input: string;
    /** Expected render / substring / marker depending on `kind`. */
    expected?: string;
    /** For `negative`: substring that must NOT appear in a successful-looking result. */
    forbidden?: string;
    notes?: string;
    /** Optional host override for this case. */
    host?: FeatureHost;
    /** Optional backend override for this case. */
    backend?: FeatureBackend;
    /** Optional device tag when `backend` is `internal-titan`. */
    device?: string;
};

export type FeatureEntry = {
    name: string;
    category: string;
    status: FeatureStatus;
    effect: FeatureEffect;
    notes?: string;
    cases: readonly FeatureCase[];
    /** Default host for cases that omit `host`. */
    host?: FeatureHost;
    /** Default backend for cases that omit `backend`. */
    backend?: FeatureBackend;
    /** Default device for cases that omit `device`. */
    device?: string;
};

export type FeatureMatrix = readonly FeatureEntry[];
