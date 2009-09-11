import { describe, expect, it } from 'vitest';
import { matlab } from '@sxo/matlab';

describe('@sxo/matlab numeric', () => {
    it('keeps exact big integers', () => {
        expect(matlab.evaluate('99999999999999999999 + 1').toMatlab()).toBe('100000000000000000000');
    });

    it('keeps exact rationals', () => {
        expect(matlab.evaluate('1/3 + 1/3 + 1/3').toMatlab()).toBe('1');
    });
});
