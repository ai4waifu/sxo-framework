/** Feature matrix shared types for `@sxo/matlab`. */

export type FeatureStatus = 'supported' | 'partial' | 'unsupported' | 'planned';

export type FeatureEffect = 'pure' | 'stateful' | 'effectful' | 'unevaluated';

export type CaseKind = 'eval' | 'parse' | 'roundtrip' | 'plot' | 'negative' | 'gap';

export type FeatureCase = {
    id: string;
    kind: CaseKind;
    input: string;
    expected?: string;
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
