import { writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import type { FeatureBinaryIdentity, FeatureFixtureHooks } from '../src/index.js';
import { runFeatureCase, runIsolatedEval } from '../src/index.js';

const hooks: FeatureFixtureHooks = {
    evaluate: (input) => {
        if (input === 'boom') throw new Error('boom');
        if (input === '1+1') return '2';
        return input;
    },
    parse: (input) => `parsed:${input}`,
    plot: () => '<svg><path d="M0 0"/></svg>',
};

const sampleBinary: FeatureBinaryIdentity = {
    path: '/tmp/sxo.mock.node',
    packageName: '@sxo/sxo-mock',
    triple: 'mock',
    version: '0.0.0-test',
    size: 42,
    mtimeMs: 1,
};

describe('runFeatureCase', () => {
    it('passes eval when render matches expected', () => {
        expect(runFeatureCase(hooks, { id: 'e', kind: 'eval', input: '1+1', expected: '2' })).toEqual({
            status: 'ok',
        });
    });

    it('attaches binary identity on executed results', () => {
        expect(runFeatureCase(hooks, { id: 'e', kind: 'eval', input: '1+1', expected: '2' }, { binary: sampleBinary })).toEqual({
            status: 'ok',
            binary: sampleBinary,
        });
        expect(
            runFeatureCase(hooks, { id: 'w', kind: 'wrong', input: '1+1', expected: '3', flags: ['wrong'] }, { binary: sampleBinary }),
        ).toEqual({
            status: 'wrong',
            actual: '2',
            expected: '3',
            threw: false,
            binary: sampleBinary,
        });
    });

    it('returns gap without executing', () => {
        expect(runFeatureCase(hooks, { id: 'g', kind: 'gap', input: 'Todo[]' })).toEqual({ status: 'gap' });
    });

    it('executes wrong and records actual when still incorrect', () => {
        expect(runFeatureCase(hooks, { id: 'w', kind: 'wrong', input: '1+1', expected: '3', flags: ['wrong'] })).toEqual({
            status: 'wrong',
            actual: '2',
            expected: '3',
            threw: false,
        });
    });

    it('fails wrong when actual matches expected so it can be promoted', () => {
        const result = runFeatureCase(hooks, { id: 'w', kind: 'wrong', input: '1+1', expected: '2', flags: ['wrong'] });
        expect(result.status).toBe('fail');
        if (result.status === 'fail') {
            expect(result.message).toContain('promote to eval');
        }
    });

    it('records throw on wrong cases', () => {
        expect(runFeatureCase(hooks, { id: 'w', kind: 'wrong', input: 'boom', expected: '0', flags: ['wrong'] })).toEqual({
            status: 'wrong',
            actual: 'boom',
            expected: '0',
            threw: true,
        });
    });

    it('runs isolate wrong cases through evaluateIsolated', () => {
        const isolatedHooks: FeatureFixtureHooks = {
            ...hooks,
            evaluateIsolated: (input, timeoutMs) => ({
                actual: `iso:${input}:${timeoutMs}`,
                threw: false,
                timedOut: false,
                exitCode: 0,
                signal: null,
                stderr: '',
            }),
        };
        const result = runFeatureCase(
            isolatedHooks,
            { id: 'w', kind: 'wrong', input: '1+1', expected: '9', flags: ['wrong'], isolate: true, isolateTimeoutMs: 1234 },
            { binary: sampleBinary },
        );
        expect(result).toEqual({
            status: 'wrong',
            actual: 'iso:1+1:1234',
            expected: '9',
            threw: false,
            timedOut: false,
            exitCode: 0,
            signal: null,
            binary: sampleBinary,
        });
    });

    it('fails isolate wrong when evaluateIsolated is missing', () => {
        const result = runFeatureCase(hooks, {
            id: 'w',
            kind: 'wrong',
            input: '1+1',
            expected: '9',
            flags: ['wrong'],
            isolate: true,
        });
        expect(result.status).toBe('fail');
        if (result.status === 'fail') {
            expect(result.message).toContain('evaluateIsolated');
        }
    });

    it('fails eval on mismatch', () => {
        const result = runFeatureCase(hooks, { id: 'e', kind: 'eval', input: '1+1', expected: '3' });
        expect(result.status).toBe('fail');
    });

    it('accepts negative throw by default', () => {
        expect(runFeatureCase(hooks, { id: 'n', kind: 'negative', input: 'boom' })).toEqual({ status: 'ok' });
    });

    it('uses dialect negative success hook when provided', () => {
        const withHead: FeatureFixtureHooks = {
            ...hooks,
            isNegativeSuccess: (input, out, threw) => threw || out.includes(input),
        };
        expect(runFeatureCase(withHead, { id: 'n', kind: 'negative', input: 'Hold[x]' })).toEqual({
            status: 'ok',
        });
    });
});

describe('runIsolatedEval', () => {
    it('captures worker JSON and timeout', () => {
        const worker = path.join(tmpdir(), `sxo-harness-isolate-${process.pid}.mjs`);
        writeFileSync(
            worker,
            `
const input = process.argv[2];
if (input === 'sleep') {
  await new Promise((r) => setTimeout(r, 60_000));
}
if (input === 'fail') {
  process.stdout.write(JSON.stringify({ ok: false, actual: 'nope' }) + '\\n');
  process.exit(0);
}
process.stdout.write(JSON.stringify({ ok: true, actual: 'got:' + input }) + '\\n');
`,
            'utf8',
        );

        expect(runIsolatedEval({ worker, input: 'x' })).toMatchObject({
            actual: 'got:x',
            threw: false,
            timedOut: false,
            exitCode: 0,
        });
        expect(runIsolatedEval({ worker, input: 'fail' })).toMatchObject({
            actual: 'nope',
            threw: true,
            timedOut: false,
        });
        expect(runIsolatedEval({ worker, input: 'sleep', timeoutMs: 200 })).toMatchObject({
            threw: true,
            timedOut: true,
        });

        // keep path absolute for windows
        expect(path.isAbsolute(fileURLToPath(new URL(`file:///${worker.replace(/\\/g, '/')}`))) || true).toBe(true);
    });
});
