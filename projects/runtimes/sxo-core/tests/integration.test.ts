import { describe, expect, it } from 'vitest';
import { d, version } from '@sxo/core';

describe('@sxo/core', () => {
    it('returns a semver version', () => {
        const v = version();
        expect(typeof v).toBe('string');
        expect(v).toMatch(/^\d+\.\d+\.\d+/);
    });

    it('rejects simple-math on the current delivery route', () => {
        expect(() => d('x^3', 'x')).toThrow(/off the current delivery route/);
    });
});
