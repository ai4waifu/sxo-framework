import { CAPABILITIES, createSession, version } from '@sxo/core';
import { describe, expect, it } from 'vitest';

describe('@sxo/core', () => {
    it('returns a semver version', () => {
        const v = version();
        expect(typeof v).toBe('string');
        expect(v).toMatch(/^\d+\.\d+\.\d+/);
    });

    it('exposes native capabilities without WASM', () => {
        expect(CAPABILITIES.host).toBe('native');
        expect(CAPABILITIES.wasm).toBe(false);
        expect(CAPABILITIES.jupyter).toBe(true);
    });

    it('creates a Session with the same version()', () => {
        const session = createSession();
        expect(session.version()).toBe(version());
    });
});
