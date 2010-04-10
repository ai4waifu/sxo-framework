import type { FeatureEntry, FeatureMatrix, FeatureStatus } from '../matrix/types.js';

function countByStatus(matrix: FeatureMatrix): Record<FeatureStatus, number> {
    const out: Record<FeatureStatus, number> = {
        supported: 0,
        partial: 0,
        unsupported: 0,
        planned: 0,
    };
    for (const e of matrix) out[e.status] += 1;
    return out;
}

/** Markdown summary table for stdout / local viewing. */
export function toMarkdownTable(matrix: FeatureMatrix): string {
    const counts = countByStatus(matrix);
    const lines: string[] = [
        `# Feature matrix (${matrix.length} entries)`,
        '',
        `| status | count |`,
        `| --- | ---: |`,
        `| supported | ${counts.supported} |`,
        `| partial | ${counts.partial} |`,
        `| unsupported | ${counts.unsupported} |`,
        `| planned | ${counts.planned} |`,
        '',
        `| name | category | status | effect | cases | notes |`,
        `| --- | --- | --- | --- | ---: | --- |`,
    ];
    for (const e of matrix) {
        const notes = (e.notes ?? '').replace(/\|/g, '\\|');
        lines.push(`| \`${e.name}\` | ${e.category} | ${e.status} | ${e.effect} | ${e.cases.length} | ${notes} |`);
    }
    return `${lines.join('\n')}\n`;
}

/** Flat rows for `console.table`. */
export function toConsoleRows(matrix: FeatureMatrix): Array<Record<string, string | number>> {
    return matrix.map((e) => ({
        name: e.name,
        category: e.category,
        status: e.status,
        effect: e.effect,
        cases: e.cases.length,
        notes: e.notes ?? '',
    }));
}

export type FeatureGapRow = {
    name: string;
    id: string;
    input: string;
    notes: string;
};

export type FeatureFlaggedRow = {
    name: string;
    id: string;
    kind: string;
    input: string;
    expected: string;
    flags: string;
    notes: string;
};

/** Gap cases for upstream handoff lists. */
export function listGaps(matrix: FeatureMatrix): FeatureGapRow[] {
    const out: FeatureGapRow[] = [];
    for (const e of matrix) {
        for (const c of e.cases) {
            if (c.kind !== 'gap') continue;
            out.push({
                name: e.name,
                id: c.id,
                input: c.input,
                notes: c.notes ?? e.notes ?? e.status,
            });
        }
    }
    return out;
}

/** Cases carrying any of the requested flags (e.g. `wrong`, `upstream-athena`). */
export function listCasesByFlags(matrix: FeatureMatrix, flags: readonly string[]): FeatureFlaggedRow[] {
    const want = new Set(flags);
    const out: FeatureFlaggedRow[] = [];
    for (const e of matrix) {
        for (const c of e.cases) {
            const caseFlags = c.flags ?? [];
            if (!caseFlags.some((f) => want.has(f))) continue;
            out.push({
                name: e.name,
                id: c.id,
                kind: c.kind,
                input: c.input,
                expected: c.expected ?? '',
                flags: caseFlags.join(','),
                notes: c.notes ?? e.notes ?? e.status,
            });
        }
    }
    return out;
}

/** Convenience: all `kind: 'wrong'` / flag `wrong` rows. */
export function listWrongs(matrix: FeatureMatrix): FeatureFlaggedRow[] {
    return listCasesByFlags(matrix, ['wrong']);
}

export function summarizeMatrix(matrix: FeatureMatrix): {
    entries: number;
    byStatus: Record<FeatureStatus, number>;
} {
    return { entries: matrix.length, byStatus: countByStatus(matrix) };
}

/** @internal helper for typed entry lists in tests. */
export function entryNames(matrix: readonly FeatureEntry[]): string[] {
    return matrix.map((e) => e.name);
}
