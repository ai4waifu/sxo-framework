import { runFeatureCase, validateFeatureMatrix } from '@sxo/harness';
import { Matlab, featureMatrix, matlab } from '@sxo/matlab';
import { describe, expect, it } from 'vitest';

const ml = Matlab.create({ autoSimplify: true });

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
                    it.todo(`${c.id}: ${c.input}`);
                    continue;
                }

                it(`${c.id} (${c.kind})`, () => {
                    const result = runFeatureCase(
                        {
                            evaluate: (input) => ml.evaluate(input).toMatlab(),
                            parse: (input) => matlab.parse(input).toMatlab(),
                            plot: (input) => ml.plot(input),
                        },
                        c,
                    );
                    expect(result.status, result.status === 'fail' ? result.message : undefined).toBe('ok');
                });
            }
        });
    }
});
