import { type IsolatedEvalResult, runIsolatedEval } from './isolate.js';
import type { CaseKind, FeatureBinaryIdentity, FeatureCase } from './types.js';

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
    /**
     * Optional child-process evaluate for `isolate: true` cases.
     * Prefer harness `runIsolatedEval` wired to a dialect worker.
     */
    evaluateIsolated?(input: string, timeoutMs: number): IsolatedEvalResult;
};

export type FeatureCaseRunOptions = {
    /** Native / host binary fingerprint recorded on executed results. */
    binary?: FeatureBinaryIdentity;
};

export type FeatureCaseRunOk = { status: 'ok'; binary?: FeatureBinaryIdentity };
export type FeatureCaseRunGap = { status: 'gap' };
/** Observed wrong: executed, still disagrees with expected (or threw). */
export type FeatureCaseRunWrong = {
    status: 'wrong';
    actual: string;
    expected?: string;
    threw: boolean;
    timedOut?: boolean;
    exitCode?: number | null;
    signal?: string | null;
    binary?: FeatureBinaryIdentity;
};
export type FeatureCaseRunFail = { status: 'fail'; message: string; binary?: FeatureBinaryIdentity };

export type FeatureCaseRunResult = FeatureCaseRunOk | FeatureCaseRunGap | FeatureCaseRunWrong | FeatureCaseRunFail;

function withBinary<T extends FeatureCaseRunResult>(result: T, binary?: FeatureBinaryIdentity): T {
    if (binary === undefined) return result;
    return { ...result, binary };
}

function fail(message: string, binary?: FeatureBinaryIdentity): FeatureCaseRunFail {
    return withBinary({ status: 'fail', message } as FeatureCaseRunFail, binary);
}

function assertExpected(_kind: CaseKind, expected: string | undefined): expected is string {
    return expected !== undefined;
}

/**
 * Run one matrix case against dialect hooks.
 * Does not talk to Vitest — callers map results to `expect` / `it.todo`.
 *
 * `gap` stays declarative (no execute). `wrong` always executes and records
 * `actual` / `expected` / `threw` (plus binary identity when provided). Matching
 * `expected` fails so the case can be promoted to `eval`. Cases with
 * `isolate: true` require `hooks.evaluateIsolated`.
 */
export function runFeatureCase(hooks: FeatureFixtureHooks, c: FeatureCase, opts: FeatureCaseRunOptions = {}): FeatureCaseRunResult {
    const binary = opts.binary;

    if (c.kind === 'gap') return { status: 'gap' };

    if (c.kind === 'wrong') {
        let actual = '';
        let threw = false;
        let timedOut: boolean | undefined;
        let exitCode: number | null | undefined;
        let signal: string | null | undefined;

        if (c.isolate) {
            if (!hooks.evaluateIsolated) {
                return fail(`wrong case \`${c.id}\` sets isolate but hooks.evaluateIsolated is missing`, binary);
            }
            const isolated = hooks.evaluateIsolated(c.input, c.isolateTimeoutMs ?? 8000);
            actual = isolated.actual;
            threw = isolated.threw;
            timedOut = isolated.timedOut;
            exitCode = isolated.exitCode;
            signal = isolated.signal;
        } else {
            try {
                actual = hooks.evaluate(c.input);
            } catch (e) {
                threw = true;
                actual = e instanceof Error ? e.message : String(e);
            }
        }

        if (c.expected !== undefined && !threw && actual === c.expected) {
            return fail(`wrong case \`${c.id}\` now matches expected ${JSON.stringify(c.expected)} — promote to eval`, binary);
        }
        return withBinary(
            {
                status: 'wrong',
                actual,
                expected: c.expected,
                threw,
                ...(timedOut !== undefined ? { timedOut } : {}),
                ...(exitCode !== undefined ? { exitCode } : {}),
                ...(signal !== undefined ? { signal } : {}),
            },
            binary,
        );
    }

    if (c.kind === 'eval') {
        if (!assertExpected(c.kind, c.expected)) {
            return fail(`eval case \`${c.id}\` is missing \`expected\``, binary);
        }
        const got = hooks.evaluate(c.input);
        if (got !== c.expected) {
            return fail(`eval \`${c.id}\`: expected ${JSON.stringify(c.expected)}, got ${JSON.stringify(got)}`, binary);
        }
        return withBinary({ status: 'ok' }, binary);
    }

    if (c.kind === 'parse' || c.kind === 'roundtrip') {
        const rendered = hooks.parse(c.input);
        const needle = c.expected ?? '';
        if (!rendered.includes(needle)) {
            return fail(`${c.kind} \`${c.id}\`: render ${JSON.stringify(rendered)} does not contain ${JSON.stringify(needle)}`, binary);
        }
        return withBinary({ status: 'ok' }, binary);
    }

    if (c.kind === 'plot') {
        if (!hooks.plot) {
            return fail(`plot case \`${c.id}\` requires a \`plot\` hook`, binary);
        }
        const svg = hooks.plot(c.input);
        // Structural gate only — style pass still needs SVG→PNG visual review.
        if (!svg.includes('<svg')) {
            return fail(`plot \`${c.id}\`: missing <svg>`, binary);
        }
        if (!(svg.includes('<polyline') || svg.includes('<path'))) {
            return fail(`plot \`${c.id}\`: missing polyline/path geometry`, binary);
        }
        if (c.expected && !svg.includes(c.expected)) {
            return fail(`plot \`${c.id}\`: missing expected substring ${JSON.stringify(c.expected)}`, binary);
        }
        return withBinary({ status: 'ok' }, binary);
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
                return fail(`negative \`${c.id}\`: forbidden substring ${JSON.stringify(c.forbidden)} appeared`, binary);
            }
            return withBinary({ status: 'ok' }, binary);
        }
        const okNegative = hooks.isNegativeSuccess ? hooks.isNegativeSuccess(c.input, out, threw) : threw;
        if (!okNegative) {
            return fail(`negative \`${c.id}\`: did not satisfy dialect negative success rule`, binary);
        }
        return withBinary({ status: 'ok' }, binary);
    }

    return fail(`unsupported case kind`, binary);
}

export type { IsolatedEvalResult, IsolatedEvalSpec } from './isolate.js';
export { runIsolatedEval };
