/** Feature matrix entry types (shape aligned with `@sxo/harness` contracts). */

export type FeatureStatus = 'supported' | 'partial' | 'unsupported' | 'planned';

export type FeatureEffect = 'pure' | 'stateful' | 'effectful' | 'unevaluated';

export type CaseKind = 'eval' | 'parse' | 'roundtrip' | 'plot' | 'negative' | 'gap';

export type FeatureCase = {
    id: string;
    kind: CaseKind;
    input: string;
    /** Expected render / substring / marker depending on `kind`. */
    expected?: string;
    /** For `negative`: substring that must NOT appear in a successful-looking result. */
    forbidden?: string;
    notes?: string;
};

export type FeatureEntry = {
    name: string;
    category: string;
    status: FeatureStatus;
    effect: FeatureEffect;
    notes?: string;
    cases: readonly FeatureCase[];
};
