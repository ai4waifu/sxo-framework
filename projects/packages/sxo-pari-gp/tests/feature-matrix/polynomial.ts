import { feature } from '@sxo/harness';

/** Univariate polynomial surface — planned until polynomial domain + oak land. */
export const polynomialFeatures = [
    feature('Pol', 'polynomial')
        .planned('Pol([coeffs]) constructor')
        .pure()
        .gap('pol.x2m1', 'Pol([ -1, 0, 1 ])', { expected: 'x^2 - 1', notes: 'render style TBD with GpForm' })
        .done(),
    feature('polfactor', 'polynomial')
        .planned('polynomial factorization')
        .pure()
        .gap('polfactor.x2m1', 'polfactor(x^2 - 1)', {
            expected: '[x - 1, x + 1]',
            notes: 'GP factoring output shape TBD',
        })
        .done(),
];
