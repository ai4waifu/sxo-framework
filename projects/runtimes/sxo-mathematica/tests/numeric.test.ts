import { describe, expect, it } from 'vitest';
import { mathematica } from '@sxo/mathematica';

describe('@sxo/mathematica numeric', () => {
    it('keeps exact big integers', () => {
        const big = mathematica.evaluate('99999999999999999999 + 1');
        expect(big.toWolfram()).toBe('100000000000000000000');
    });

    it('keeps exact rationals', () => {
        const exact = mathematica.evaluate('1/3 + 1/3 + 1/3');
        expect(exact.toWolfram()).toBe('1');
    });
});
