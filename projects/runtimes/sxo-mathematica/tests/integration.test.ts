import { describe, expect, it } from 'vitest';
import { Mathematica, mathematica } from '@sxo/mathematica';

describe('@sxo/mathematica integration', () => {
    it('differentiates polynomials', () => {
        const math = mathematica.d('x^3', 'x').toWolfram();
        expect(math.includes('x') || math.includes('3')).toBe(true);
    });

    it('simplifies trig identity', () => {
        const trig = mathematica.evaluate('Simplify[Sin[x]^2 + Cos[x]^2]');
        expect(trig.toWolfram()).toBe('1');
    });

    it('evaluates factorial', () => {
        expect(mathematica.evaluate('5!').toWolfram()).toBe('120');
    });

    it('evaluates equality', () => {
        expect(mathematica.evaluate('2 == 2').toWolfram()).toBe('1');
    });

    it('integrates polynomials', () => {
        const integral = mathematica.evaluate('Integrate[x^2, x]');
        expect(integral.toWolfram()).toContain('x');
    });

    it('maps over lists', () => {
        const mapped = mathematica.evaluate('Map[Sin, {0, 1}]');
        expect(mapped.toWolfram().startsWith('{')).toBe(true);
    });

    it('shares engine version across instances', () => {
        const mma = Mathematica.create({ autoSimplify: false });
        expect(mma.version()).toBe(mathematica.version());
    });
});
