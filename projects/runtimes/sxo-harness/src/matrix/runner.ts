import type { CaseKind, FeatureCase } from './types.js';

/** Dialect-facing render hooks for internal fixture runs. */
export type FeatureFixtureHooks = {
    /** Evaluate then render to the dialect surface string. */
    evaluate(input: string): string;
    /** Parse then render (no evaluate). */
    parse(input: string): string;
    /** Optional plot SVG renderer. */
    plot?(input: string): string;
    /**
     * Optional dialect rule when `negative` has no `forbidden`.
     * Default requires a thrown error.
     */
    isNegativeSuccess?(input: string, out: string, threw: boolean): boolean;
};

export type FeatureCaseRunOk = { status: 'ok' };
export type FeatureCaseRunGap = { status: 'gap' };
export type FeatureCaseRunFail = { status: 'fail'; message: string };

export type FeatureCaseRunResult = FeatureCaseRunOk | FeatureCaseRunGap | FeatureCaseRunFail;

function fail(message: string): FeatureCaseRunFail {
    return { status: 'fail', message };
}

function assertExpected(kind: CaseKind, expected: string | undefined): expected is string {
    return expected !== undefined;
}

/**
 * Run one matrix case against dialect hooks.
 * Does not talk to Vitest — callers map results to `expect` / `it.todo`.
 */
export function runFeatureCase(hooks: FeatureFixtureHooks, c: FeatureCase): FeatureCaseRunResult {
    if (c.kind === 'gap') return { status: 'gap' };

    if (c.kind === 'eval') {
        if (!assertExpected(c.kind, c.expected)) {
            return fail(`eval case \`${c.id}\` is missing \`expected\``);
        }
        const got = hooks.evaluate(c.input);
        if (got !== c.expected) {
            return fail(`eval \`${c.id}\`: expected ${JSON.stringify(c.expected)}, got ${JSON.stringify(got)}`);
        }
        return { status: 'ok' };
    }

    if (c.kind === 'parse' || c.kind === 'roundtrip') {
        const rendered = hooks.parse(c.input);
        const needle = c.expected ?? '';
        if (!rendered.includes(needle)) {
            return fail(
                `${c.kind} \`${c.id}\`: render ${JSON.stringify(rendered)} does not contain ${JSON.stringify(needle)}`,
            );
        }
        return { status: 'ok' };
    }

    if (c.kind === 'plot') {
        if (!hooks.plot) {
            return fail(`plot case \`${c.id}\` requires a \`plot\` hook`);
        }
        const svg = hooks.plot(c.input);
        // Structural gate only — style pass still needs SVG→PNG visual review.
        if (!svg.includes('<svg')) {
            return fail(`plot \`${c.id}\`: missing <svg>`);
        }
        if (!(svg.includes('<polyline') || svg.includes('<path'))) {
            return fail(`plot \`${c.id}\`: missing polyline/path geometry`);
        }
        if (c.expected && !svg.includes(c.expected)) {
            return fail(`plot \`${c.id}\`: missing expected substring ${JSON.stringify(c.expected)}`);
        }
        return { status: 'ok' };
    }

    if (c.kind === 'negative') {
        let threw = false;
        let out = '';
        try {
            out = hooks.evaluate(c.input);
        } catch {
            threw = true;
        }
        if (c.forbidden) {
            if (!(threw || !out.includes(c.forbidden))) {
                return fail(`negative \`${c.id}\`: forbidden substring ${JSON.stringify(c.forbidden)} appeared`);
            }
            return { status: 'ok' };
        }
        const okNegative = hooks.isNegativeSuccess
            ? hooks.isNegativeSuccess(c.input, out, threw)
            : threw;
        if (!okNegative) {
            return fail(`negative \`${c.id}\`: did not satisfy dialect negative success rule`);
        }
        return { status: 'ok' };
    }

    return fail(`unsupported case kind`);
}
