import { fileURLToPath } from 'node:url';
import { nativeBinaryIdentity } from '@sxo/core';
import { runFeatureCase, runIsolatedEval, validateFeatureMatrix } from '@sxo/harness';
import { Matlab, matlab } from '@sxo/matlab';
import { describe, expect, it } from 'vitest';
import { featureMatrix } from './feature-matrix/index.js';

const ml = Matlab.create({ autoSimplify: true });
const binary = nativeBinaryIdentity();
const isolateWorker = fileURLToPath(new URL('./isolate-eval.mjs', import.meta.url));

const hooks = {
    evaluate: (input: string) => ml.evaluate(input).toMatlab(),
    parse: (input: string) => matlab.parse(input).toMatlab(),
    plot: (input: string) => ml.plot(input),
    evaluateIsolated: (input: string, timeoutMs: number) =>
        runIsolatedEval({
            worker: isolateWorker,
            input,
            timeoutMs,
        }),
};

describe('@sxo/matlab feature matrix', () => {
    it('passes shared harness status rules', () => {
        const result = validateFeatureMatrix(featureMatrix);
        expect(result.issues, result.issues.map((i) => i.message).join('\n')).toEqual([]);
        expect(result.ok).toBe(true);
    });

    it('records native binary identity', () => {
        expect(binary.path).toContain('sxo.');
        expect(binary.version.length).toBeGreaterThan(0);
        expect(binary.size).toBeGreaterThan(0);
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
                        const result = runFeatureCase(hooks, c, { binary });
                        if (result.status === 'fail') {
                            expect.fail(result.message);
                        }
                        expect(result.status).toBe('wrong');
                        if (result.status === 'wrong') {
                            expect(result.binary?.path).toBe(binary.path);
                            expect.soft(result.actual, `expected=${JSON.stringify(result.expected)} threw=${result.threw}`).toBeDefined();
                        }
                    });
                    continue;
                }

                it(`${c.id} (${c.kind})`, () => {
                    const result = runFeatureCase(hooks, c, { binary });
                    expect(result.status, result.status === 'fail' ? result.message : undefined).toBe('ok');
                });
            }
        });
    }
});
