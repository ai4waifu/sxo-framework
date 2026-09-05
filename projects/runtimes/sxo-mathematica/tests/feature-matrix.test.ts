import { runFeatureCase, validateFeatureMatrix } from '@sxo/harness';
import { Mathematica, featureMatrix, mathematica } from '@sxo/mathematica';
import { describe, expect, it } from 'vitest';

const mma = Mathematica.create({ autoSimplify: true });

describe('@sxo/mathematica feature matrix', () => {
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
                            evaluate: (input) => mma.evaluate(input).toWolfram(),
                            parse: (input) => mathematica.parse(input).toWolfram(),
                            plot: (input) => mma.plot(input),
                            isNegativeSuccess: (input, out, threw) =>
                                threw || out.includes(input.split('[')[0] ?? input),
                        },
                        c,
                    );
                    expect(result.status, result.status === 'fail' ? result.message : undefined).toBe('ok');
                });
            }
        });
    }
});
