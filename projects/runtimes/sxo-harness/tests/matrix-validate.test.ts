import { describe, expect, it } from 'vitest';
import type { FeatureEntry } from '../src/index.js';
import { validateFeatureMatrix } from '../src/index.js';

function entry(partial: Partial<FeatureEntry> & Pick<FeatureEntry, 'name' | 'status' | 'cases'>): FeatureEntry {
    return {
        category: 'test',
        effect: 'pure',
        ...partial,
    };
}

describe('validateFeatureMatrix', () => {
    it('accepts a supported entry with a runnable case', () => {
        const result = validateFeatureMatrix([
            entry({
                name: 'Plus',
                status: 'supported',
                cases: [{ id: 'plus.1', kind: 'eval', input: '1+1', expected: '2' }],
            }),
        ]);
        expect(result.ok).toBe(true);
        expect(result.issues).toEqual([]);
    });

    it('rejects supported without non-gap cases', () => {
        const result = validateFeatureMatrix([
            entry({
                name: 'Map',
                status: 'supported',
                cases: [{ id: 'map.gap', kind: 'gap', input: 'Map[f,{1}]' }],
            }),
        ]);
        expect(result.ok).toBe(false);
        expect(result.issues.some((i) => i.code === 'supported_without_runnable_case')).toBe(true);
    });

    it('rejects planned entries with non-gap cases', () => {
        const result = validateFeatureMatrix([
            entry({
                name: 'Future',
                status: 'planned',
                cases: [{ id: 'future.1', kind: 'eval', input: 'Future[]', expected: '1' }],
            }),
        ]);
        expect(result.ok).toBe(false);
        expect(result.issues.some((i) => i.code === 'planned_non_gap_case')).toBe(true);
    });

    it('rejects duplicate case ids across entries', () => {
        const result = validateFeatureMatrix([
            entry({
                name: 'A',
                status: 'partial',
                cases: [{ id: 'shared', kind: 'gap', input: 'a' }],
            }),
            entry({
                name: 'B',
                status: 'partial',
                cases: [{ id: 'shared', kind: 'gap', input: 'b' }],
            }),
        ]);
        expect(result.ok).toBe(false);
        expect(result.issues.some((i) => i.code === 'duplicate_case_id')).toBe(true);
    });
});
