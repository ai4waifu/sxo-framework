import { runFeatureCase, validateFeatureMatrix } from '@sxo/harness';
import { Matlab, matlab } from '@sxo/matlab';
import { describe, expect, it } from 'vitest';
import { featureMatrix } from './feature-matrix/index.js';

const ml = Matlab.create({ autoSimplify: true });

const hooks = {
    evaluate: (input: string) => ml.evaluate(input).toMatlab(),
    parse: (input: string) => matlab.parse(input).toMatlab(),
    plot: (input: string) => ml.plot(input),
};

describe('@sxo/matlab feature matrix', () => {
    it('passes shared harness status rules', () => {
        const result = validateFeatureMatrix(featureMatrix);
        expect(result.issues, result.issues.map((i) => i.message).join('\n')).toEqual([]);
        expect(result.ok).toBe(true);
    });

    for (const entry of featureMatrix) {
        describe(`${entry.name} [${entry.status}]`, () => {
            for (const c of entry.cases) {
                if (c.kind === 'gap') {
                    const flagNote = c.flags?.length ? ` [${c.flags.join(',')}]` : '';
                    it.todo(`${c.id}: ${c.input}${flagNote}`);
                    continue;
                }

                if (c.kind === 'wrong') {
                    it(`${c.id} (wrong diagnostic)`, () => {
                        const result = runFeatureCase(hooks, c);
                        if (result.status === 'fail') {
                            expect.fail(result.message);
                        }
                        expect(result.status).toBe('wrong');
                        if (result.status === 'wrong') {
                            // Force observation into the vitest report without failing CI.
                            expect.soft(result.actual, `expected=${JSON.stringify(result.expected)} threw=${result.threw}`).toBeDefined();
                        }
                    });
                    continue;
                }

                it(`${c.id} (${c.kind})`, () => {
                    const result = runFeatureCase(hooks, c);
                    expect(result.status, result.status === 'fail' ? result.message : undefined).toBe('ok');
                });
            }
        });
    }
});
