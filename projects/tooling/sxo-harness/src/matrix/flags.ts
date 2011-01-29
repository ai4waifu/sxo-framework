/**
 * Well-known triage flags for feature-matrix cases.
 *
 * Free-form strings remain allowed on `FeatureCase.flags`. Prefer these names so
 * reports stay filterable (`report:features --flag=…` / `wrongs` / `suboptimal`).
 *
 * Semantics:
 * - `wrong` — known incorrect / upstream-broken hole (`kind: 'wrong'`, Vitest todo)
 * - `suboptimal` — current lock is acceptable but not ideal (usually runnable `eval`)
 * - `cosmetic` — render / presentation only, semantics OK
 * - `upstream-athena` / `upstream-oaks` / `upstream-apollo` — ownership for handoff
 */
export const FEATURE_CASE_FLAGS = {
    wrong: 'wrong',
    suboptimal: 'suboptimal',
    cosmetic: 'cosmetic',
    upstreamAthena: 'upstream-athena',
    upstreamOaks: 'upstream-oaks',
    upstreamApollo: 'upstream-apollo',
} as const;

export type WellKnownFeatureCaseFlag = (typeof FEATURE_CASE_FLAGS)[keyof typeof FEATURE_CASE_FLAGS];

/** Merge auto flags with caller extras (stable unique order: autos then extras). */
export function mergeCaseFlags(auto: readonly string[], extras?: readonly string[]): string[] {
    const out: string[] = [];
    const seen = new Set<string>();
    for (const f of [...auto, ...(extras ?? [])]) {
        if (!f || seen.has(f)) continue;
        seen.add(f);
        out.push(f);
    }
    return out;
}
