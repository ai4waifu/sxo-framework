import { describe, expect, it } from 'vitest';
import type { FeatureFixtureHooks } from '../src/index.js';
import { runFeatureCase } from '../src/index.js';

const hooks: FeatureFixtureHooks = {
    evaluate: (input) => {
        if (input === 'boom') throw new Error('boom');
        if (input === '1+1') return '2';
        return input;
    },
    parse: (input) => `parsed:${input}`,
    plot: () => '<svg><path d="M0 0"/></svg>',
};

describe('runFeatureCase', () => {
    it('passes eval when render matches expected', () => {
        expect(runFeatureCase(hooks, { id: 'e', kind: 'eval', input: '1+1', expected: '2' })).toEqual({
            status: 'ok',
        });
    });

    it('returns gap without executing', () => {
        expect(runFeatureCase(hooks, { id: 'g', kind: 'gap', input: 'Todo[]' })).toEqual({ status: 'gap' });
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
