import { CAPABILITIES, capabilities } from '@sxo/lite';
import { describe, expect, it } from 'vitest';

describe('@sxo/lite', () => {
    it('exposes WASM capabilities without Jupyter', () => {
        expect(CAPABILITIES.host).toBe('wasm');
        expect(CAPABILITIES.wasm).toBe(true);
        expect(CAPABILITIES.jupyter).toBe(false);
        expect(capabilities()).toEqual(CAPABILITIES);
    });
});
