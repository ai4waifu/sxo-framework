import { validateFeatureMatrix } from '@sxo/harness';
import { describe, expect, it } from 'vitest';
import { PARI_GP_DIALECT, packageName } from '../src/index.js';
import { featureMatrix } from './feature-matrix/index.js';

describe('@sxo/pari-gp', () => {
    it('exports dialect identity', () => {
        expect(packageName()).toBe('@sxo/pari-gp');
        expect(PARI_GP_DIALECT).toBe('pari-gp');
    });

    it('passes shared harness status rules', () => {
        const result = validateFeatureMatrix(featureMatrix);
        expect(result.issues, result.issues.map((i) => i.message).join('\n')).toEqual([]);
        expect(result.ok).toBe(true);
    });

    for (const entry of featureMatrix) {
        describe(`${entry.name} [${entry.status}]`, () => {
            for (const c of entry.cases) {
                if (c.kind === 'gap' || c.kind === 'wrong') {
                    const flagNote = c.flags?.length ? ` [${c.flags.join(',')}]` : '';
                    // PARI/GP has no GpForm evaluate hooks yet — keep declarative until R-2.17+.
                    it.todo(`${c.id}: ${c.input}${flagNote}`);
                    continue;
                }
                it.todo(`${c.id}: runnable cases require GpForm evaluate hooks`);
            }
        });
    }
});
