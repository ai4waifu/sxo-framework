import { describe, expect, it } from 'vitest';
import type { FeatureMatrix } from '../src/index.js';
import { listGaps, summarizeMatrix, toMarkdownTable } from '../src/index.js';

const sample: FeatureMatrix = [
    {
        name: 'Plus',
        category: 'arithmetic',
        status: 'supported',
        effect: 'pure',
        cases: [{ id: 'plus.1', kind: 'eval', input: '1+1', expected: '2' }],
    },
    {
        name: 'Map',
        category: 'list',
        status: 'partial',
        effect: 'pure',
        notes: 'subset',
        cases: [{ id: 'map.gap', kind: 'gap', input: 'Map[f,{1}]', notes: 'symbolic' }],
    },
];

describe('matrix reporters', () => {
    it('summarizes status counts', () => {
        expect(summarizeMatrix(sample)).toEqual({
            entries: 2,
            byStatus: { supported: 1, partial: 1, unsupported: 0, planned: 0 },
        });
    });

    it('lists gap rows', () => {
        expect(listGaps(sample)).toEqual([
            { name: 'Map', id: 'map.gap', input: 'Map[f,{1}]', notes: 'symbolic' },
        ]);
    });

    it('renders a markdown table with counts', () => {
        const md = toMarkdownTable(sample);
        expect(md).toContain('| supported | 1 |');
        expect(md).toContain('| `Plus` |');
        expect(md).toContain('| `Map` |');
    });
});
