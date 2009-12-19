import { Mathematica, mathematica } from '@sxo/mathematica';
import { describe, expect, it } from 'vitest';

describe('@sxo/mathematica integration', () => {
    it('differentiates polynomials', () => {
        const math = mathematica.d('x^3', 'x');
        expect(math.toWolfram()).toBe('3*x^2');
    });

    it('simplifies trig identity', () => {
        const trig = mathematica.evaluate('Simplify[Sin[x]^2 + Cos[x]^2]');
        expect(trig.toWolfram()).toBe('1');
    });

    it('evaluates factorial', () => {
        expect(mathematica.evaluate('5!').toWolfram()).toBe('120');
        expect(mathematica.evaluate('50!').toWolfram()).toBe('30414093201713378043612608166064768844377641568960512000000000000');
    });

    it('evaluates equality', () => {
        expect(mathematica.evaluate('2 == 2').toWolfram()).toBe('True');
    });

    it('integrates polynomials', () => {
        const integral = mathematica.evaluate('Integrate[x^2, x]');
        expect(integral.toWolfram()).toBe('1/3*x^3');
    });

    it('maps slot pure functions over lists', () => {
        const mapped = mathematica.evaluate('Map[#^2 &, {1, 2, 3}]');
        expect(mapped.toWolfram()).toBe('{1, 4, 9}');
    });

    it('shares engine version across instances', () => {
        const mma = Mathematica.create({ autoSimplify: false });
        expect(mma.version()).toBe(mathematica.version());
    });
});
