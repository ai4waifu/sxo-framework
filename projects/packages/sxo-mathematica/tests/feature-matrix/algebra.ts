import { feature } from '@sxo/harness';

export const algebraFeatures = [
    feature('Expand', 'algebra')
        .partial('exact `Expand[(x + 1)^2]` via binomial square kernel')
        .pure()
        .eval('expand.bin', 'Expand[(x + 1)^2]', '1 + 2*x + x^2')
        .done(),
    feature('Factor', 'algebra')
        .partial('exact `Factor[x^2 - 1]` via difference-of-squares kernel')
        .pure()
        .eval('factor.diff', 'Factor[x^2 - 1]', '(-1 + x)*(1 + x)')
        .done(),
    feature('Collect', 'algebra')
        .partial('identity `Collect` when expression already collected w.r.t. variable')
        .pure()
        .eval('collect.xy', 'Collect[x^2 + 2*x*y + y^2, x]', 'x^2 + 2*x*y + y^2')
        .done(),
    feature('Cancel', 'algebra')
        .partial('exact `Cancel[(x^2 - 1)/(x - 1)]` via rational cancel kernel')
        .pure()
        .eval('cancel.x2m1', 'Cancel[(x^2 - 1)/(x - 1)]', '1 + x')
        .done(),
    feature('Variables', 'algebra')
        .partial('exact `Variables[x*y + z]` via symbol walk')
        .pure()
        .eval('variables.xyz', 'Variables[x*y + z]', '{x, y, z}')
        .done(),
    feature('Numerator', 'algebra').partial('exact `Numerator` on tested rationals and integers').pure().eval('numerator.half', 'Numerator[1/2]', '1').eval('numerator.int', 'Numerator[5]', '5').done(),
    feature('Denominator', 'algebra').partial('exact `Denominator` on tested rationals and integers').pure().eval('denominator.34', 'Denominator[3/4]', '4').eval('denominator.int', 'Denominator[5]', '1').done(),
    feature('Together', 'algebra')
        .partial('exact `Together[1/x + 1/y]` via common-denominator lowering')
        .pure()
        .eval('together.xy', 'Together[1/x + 1/y]', '(x + y)/(x*y)')
        .done(),
    feature('Apart', 'algebra')
        .partial('exact `Apart[1/(x*(x + 1))]` via linear partial-fraction lowering')
        .pure()
        .eval('apart.partial', 'Apart[1/(x*(x + 1))]', '1/x - 1/(1 + x)')
        .done(),
    feature('Coefficient', 'algebra')
        .partial('exact `Coefficient[x^2 + 3*x, x]` via summand walk')
        .pure()
        .eval('coefficient.x', 'Coefficient[x^2 + 3*x, x]', '3')
        .done(),
    feature('Exponent', 'algebra')
        .partial('exact `Exponent[x^3 + x, x]` via max power walk')
        .pure()
        .eval('exponent.x3', 'Exponent[x^3 + x, x]', '3')
        .done(),
    feature('PolynomialGCD', 'algebra')
        .partial('exact `PolynomialGCD[x^2 - 1, x - 1]` via difference-of-squares kernel')
        .pure()
        .eval('polygcd.basic', 'PolynomialGCD[x^2 - 1, x - 1]', '-1 + x')
        .done(),
    feature('Discriminant', 'algebra').unsupported().pure().gap('discriminant.quad', 'Discriminant[x^2 + x + 1, x]', { expected: '-3' }).done(),
    feature('Resultant', 'algebra').unsupported().pure().gap('resultant.basic', 'Resultant[x^2 - 1, x - 1, x]', { expected: '0' }).done(),
    feature('PolynomialRemainder', 'algebra')
        .unsupported()
        .pure()
        .gap('polyrem.basic', 'PolynomialRemainder[x^3 + 1, x + 1, x]', { expected: '0' })
        .done(),
];
