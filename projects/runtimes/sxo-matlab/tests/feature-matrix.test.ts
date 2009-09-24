import { Matlab, featureMatrix, matlab } from '@sxo/matlab';
import { describe, expect, it } from 'vitest';

const ml = Matlab.create({ autoSimplify: true });

describe('@sxo/matlab feature matrix', () => {
    it('requires every supported entry to have a non-gap case', () => {
        for (const entry of featureMatrix) {
            if (entry.status !== 'supported') continue;
            const runnable = entry.cases.filter((c) => c.kind !== 'gap');
            expect(runnable.length, `${entry.name} supported without cases`).toBeGreaterThan(0);
        }
    });

    it('keeps case ids unique', () => {
        const ids = featureMatrix.flatMap((e) => e.cases.map((c) => c.id));
        expect(new Set(ids).size).toBe(ids.length);
    });

    for (const entry of featureMatrix) {
        describe(`${entry.name} [${entry.status}]`, () => {
            for (const c of entry.cases) {
                if (c.kind === 'gap') {
                    it.todo(`${c.id}: ${c.input}`);
                    continue;
                }

                it(`${c.id} (${c.kind})`, () => {
                    if (c.kind === 'eval') {
                        expect(ml.evaluate(c.input).toMatlab()).toBe(c.expected);
                        return;
                    }
                    if (c.kind === 'parse' || c.kind === 'roundtrip') {
                        const rendered = matlab.parse(c.input).toMatlab();
                        expect(rendered).toContain(c.expected ?? '');
                        return;
                    }
                    if (c.kind === 'plot') {
                        const svg = ml.plot(c.input);
                        expect(svg).toContain('<svg');
                        expect(svg.includes('<polyline') || svg.includes('<path')).toBe(true);
                        if (c.expected) expect(svg).toContain(c.expected);
                        return;
                    }
                    if (c.kind === 'negative') {
                        let threw = false;
                        let out = '';
                        try {
                            out = ml.evaluate(c.input).toMatlab();
                        } catch {
                            threw = true;
                        }
                        if (c.forbidden) {
                            expect(threw || !out.includes(c.forbidden)).toBe(true);
                        } else {
                            expect(threw).toBe(true);
                        }
                    }
                });
            }
        });
    }
});
