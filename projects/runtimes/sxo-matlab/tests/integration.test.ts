import { Matlab, matlab } from '@sxo/matlab';
import { describe, expect, it } from 'vitest';

describe('@sxo/matlab integration', () => {
    it('differentiates polynomials', () => {
        const ml = matlab.d('x^3', 'x');
        expect(ml.toMatlab()).toBe('3*x^2');
    });

    it('simplifies trig identity', () => {
        expect(matlab.evaluate('sin(x)^2 + cos(x)^2').toMatlab()).toBe('1');
    });

    it('evaluates multi-statement input', () => {
        expect(matlab.evaluate('1; 2+2').toMatlab()).toBe('4');
    });

    it('parses matrices', () => {
        expect(matlab.parse('[1, 2; 3, 4]').toMatlab()).toBe('[1, 2; 3, 4]');
    });

    it('integrates polynomials', () => {
        expect(matlab.evaluate('int(x^2, x)').toMatlab()).toBe('1/3*x^3');
    });

    it('evaluates comparisons', () => {
        expect(matlab.evaluate('3 > 2').toMatlab()).toBe('1');
    });

    it('shares engine version across instances', () => {
        const mlInst = Matlab.create({ autoSimplify: false });
        expect(mlInst.version()).toBe(matlab.version());
    });
});
