import { validateFeatureMatrix } from '@sxo/harness';
import { describe, expect, it } from 'vitest';
import { PARI_GP_DIALECT, PariGp, PariGpUnsupportedError, packageName, pariGp } from '../src/index.js';
import { featureMatrix } from './feature-matrix/index.js';

describe('@sxo/pari-gp', () => {
    it('exports dialect identity', () => {
        expect(packageName()).toBe('@sxo/pari-gp');
        expect(PARI_GP_DIALECT).toBe('pari-gp');
        expect(PariGp.create()).toBeInstanceOf(PariGp);
    });

    it('placeholder API throws until GpForm lands', () => {
        expect(() => pariGp.parse('1+1')).toThrow(PariGpUnsupportedError);
        expect(() => pariGp.evaluate('factor(6)')).toThrow(PariGpUnsupportedError);
    });

    it('passes shared harness status rules', () => {
        const result = validateFeatureMatrix(featureMatrix);
        expect(result.issues, result.issues.map((i) => i.message).join('\n')).toEqual([]);
        expect(result.ok).toBe(true);
    });

    it('covers core planned categories', () => {
        const categories = new Set(featureMatrix.map((e) => e.category));
        for (const want of ['arithmetic', 'number_theory', 'modular', 'polynomial', 'session']) {
            expect(categories.has(want), `missing category ${want}`).toBe(true);
        }
        expect(featureMatrix.every((e) => e.status === 'planned')).toBe(true);
    });

    for (const entry of featureMatrix) {
        describe(`${entry.name} [${entry.status}]`, () => {
            for (const c of entry.cases) {
                if (c.kind === 'gap' || c.kind === 'wrong') {
                    const flagNote = c.flags?.length ? ` [${c.flags.join(',')}]` : '';
                    // PARI/GP has no GpForm evaluate hooks yet — keep declarative.
                    it.todo(`${c.id}: ${c.input}${flagNote}`);
                    continue;
                }
                it.todo(`${c.id}: runnable cases require GpForm evaluate hooks`);
            }
        });
    }
});
