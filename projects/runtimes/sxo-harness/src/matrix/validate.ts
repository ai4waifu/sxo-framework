import type { FeatureEntry, FeatureMatrix } from './types.js';

export type MatrixValidationIssue = {
    code:
        | 'supported_without_runnable_case'
        | 'planned_non_gap_case'
        | 'duplicate_case_id'
        | 'empty_cases'
        | 'missing_case_id'
        | 'missing_entry_name';
    entry?: string;
    caseId?: string;
    message: string;
};

export type MatrixValidationResult = {
    ok: boolean;
    issues: MatrixValidationIssue[];
};

/**
 * Structural Feature Matrix status rules (Living harness contract).
 * Does not execute dialect fixtures — only validates declarations.
 */
export function validateFeatureMatrix(matrix: FeatureMatrix): MatrixValidationResult {
    const issues: MatrixValidationIssue[] = [];
    const seenIds = new Map<string, string>();

    for (const entry of matrix) {
        if (!entry.name.trim()) {
            issues.push({
                code: 'missing_entry_name',
                message: 'Feature entry is missing a non-empty `name`',
            });
        }

        if (entry.cases.length === 0) {
            issues.push({
                code: 'empty_cases',
                entry: entry.name,
                message: `Feature \`${entry.name}\` has no cases`,
            });
        }

        if (entry.status === 'supported') {
            const runnable = entry.cases.filter((c) => c.kind !== 'gap');
            if (runnable.length === 0) {
                issues.push({
                    code: 'supported_without_runnable_case',
                    entry: entry.name,
                    message: `Feature \`${entry.name}\` is \`supported\` but has no non-\`gap\` case`,
                });
            }
        }

        if (entry.status === 'planned') {
            for (const c of entry.cases) {
                if (c.kind !== 'gap') {
                    issues.push({
                        code: 'planned_non_gap_case',
                        entry: entry.name,
                        caseId: c.id,
                        message: `Feature \`${entry.name}\` is \`planned\` but case \`${c.id}\` has kind \`${c.kind}\` (only \`gap\` allowed)`,
                    });
                }
            }
        }

        for (const c of entry.cases) {
            if (!c.id.trim()) {
                issues.push({
                    code: 'missing_case_id',
                    entry: entry.name,
                    message: `Feature \`${entry.name}\` has a case with empty \`id\``,
                });
                continue;
            }
            const prior = seenIds.get(c.id);
            if (prior !== undefined) {
                issues.push({
                    code: 'duplicate_case_id',
                    entry: entry.name,
                    caseId: c.id,
                    message: `Case id \`${c.id}\` is duplicated (also on \`${prior}\`)`,
                });
            } else {
                seenIds.set(c.id, entry.name);
            }
        }
    }

    return { ok: issues.length === 0, issues };
}

/** Convenience helper for tests / reporters. */
export function assertValidFeatureMatrix(matrix: FeatureMatrix): void {
    const result = validateFeatureMatrix(matrix);
    if (result.ok) return;
    const detail = result.issues.map((i) => i.message).join('\n');
    throw new Error(`Invalid feature matrix:\n${detail}`);
}

export function listRunnableCases(entry: FeatureEntry): FeatureEntry['cases'] {
    return entry.cases.filter((c) => c.kind !== 'gap');
}
