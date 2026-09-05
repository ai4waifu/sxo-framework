import { feature } from '@sxo/harness';

export const algebraFeatures = [
    feature('Expand', 'algebra').unsupported().pure().gap('expand.bin', 'Expand[(x + 1)^2]', { expected: '1 + 2*x + x^2' }).done(),
    feature('Factor', 'algebra').unsupported().pure().gap('factor.diff', 'Factor[x^2 - 1]', { expected: '(-1 + x)*(1 + x)' }).done(),
    feature('Cancel', 'algebra')
        .unsupported('SILENT WRONG rewrite of (x^2-1)/(x-1) into Cancel[-((-1+x)^-1)+…]')
        .pure()
        .gap('cancel.x2m1', 'Cancel[(x^2 - 1)/(x - 1)]', { expected: '1 + x' })
        .done(),
    feature('Variables', 'algebra').unsupported().pure().gap('variables.xyz', 'Variables[x*y + z]', { expected: '{x, y, z}' }).done(),
    feature('Numerator', 'algebra').unsupported().pure().gap('numerator.half', 'Numerator[1/2]', { expected: '1' }).done(),
    feature('Denominator', 'algebra').unsupported().pure().gap('denominator.34', 'Denominator[3/4]', { expected: '4' }).done(),
    feature('Together', 'algebra').unsupported().pure().gap('together.xy', 'Together[1/x + 1/y]', { expected: '(x + y)/(x*y)' }).done(),
    feature('Apart', 'algebra').unsupported().pure().gap('apart.partial', 'Apart[1/(x*(x + 1))]', { expected: '1/x - 1/(1 + x)' }).done(),
    feature('Coefficient', 'algebra').unsupported().pure().gap('coefficient.x', 'Coefficient[x^2 + 3*x, x]', { expected: '3' }).done(),
    feature('Exponent', 'algebra').unsupported().pure().gap('exponent.x3', 'Exponent[x^3 + x, x]', { expected: '3' }).done(),
    feature('PolynomialGCD', 'algebra')
        .unsupported()
        .pure()
        .gap('polygcd.basic', 'PolynomialGCD[x^2 - 1, x - 1]', { expected: '-1 + x' })
        .done(),
    feature('Discriminant', 'algebra').unsupported().pure().gap('discriminant.quad', 'Discriminant[x^2 + x + 1, x]', { expected: '-3' }).done(),
    feature('Resultant', 'algebra').unsupported().pure().gap('resultant.basic', 'Resultant[x^2 - 1, x - 1, x]', { expected: '0' }).done(),
    feature('PolynomialRemainder', 'algebra')
        .unsupported()
        .pure()
        .gap('polyrem.basic', 'PolynomialRemainder[x^3 + 1, x + 1, x]', { expected: '0' })
        .done(),
];
