import { feature } from '@sxo/harness';

export const specialFeatures = [
    feature('Gamma', 'special')
        .partial('exact `Gamma` on tested positive integers and half')
        .pure()
        .eval('gamma.1', 'Gamma[1]', '1')
        .eval('gamma.5', 'Gamma[5]', '24')
        .done(),
    feature('Zeta', 'special').unsupported().pure().gap('zeta.2', 'Zeta[2]', { expected: 'Pi^2/6' }).done(),
    feature('Erf', 'special').partial('exact `Erf[0]` fold').pure().eval('erf.0', 'Erf[0]', '0').done(),
    feature('UnitStep', 'special')
        .partial('exact `UnitStep` on tested positive integers')
        .pure()
        .eval('unitstep.1', 'UnitStep[1]', '1')
        .done(),
    feature('UnitStepDerivative', 'special')
        .unsupported('D[UnitStep[x],x] host stack-overflow crash')
        .pure()
        .gap('unitstep.deriv_crash', 'D[UnitStep[x], x]', {
            expected: 'DiracDelta[x]',
            notes: 'host crash (stack overflow) observed — do not promote to eval',
        })
        .done(),
    feature('HeavisideTheta', 'special').partial('exact `HeavisideTheta[1]` via `UnitStep`').pure().eval('heaviside.1', 'HeavisideTheta[1]', '1').done(),
    feature('BesselJ', 'special').unsupported().pure().gap('besselj.10', 'BesselJ[1, 0]', { expected: '0' }).done(),
    feature('LegendreP', 'special').unsupported().pure().gap('legendrep.2', 'LegendreP[2, x]', { expected: '(-1 + 3*x^2)/2' }).done(),
    feature('GammaHalf', 'special').partial('exact `Gamma[1/2]` fold to `Sqrt[Pi]`').pure().eval('gamma.half', 'Gamma[1/2]', 'Sqrt[Pi]').done(),
    feature('ZetaZero', 'special').unsupported().pure().gap('zeta.0', 'Zeta[0]', { expected: '-1/2' }).done(),
    feature('Sinc', 'special')
        .partial('exact `Sinc[0]` and `Sinc[Pi]` via kernel fold')
        .pure()
        .eval('sinc.0', 'Sinc[0]', '1')
        .eval('sinc.pi', 'Sinc[Pi]', '0')
        .done(),
    feature('FresnelC', 'special').unsupported().pure().gap('fresnelc.inf', 'FresnelC[Infinity]', { expected: '1/2' }).done(),
    feature('InverseErf', 'special').unsupported().pure().gap('inverseerf.0', 'InverseErf[0]', { expected: '0' }).done(),
    feature('SquareWave', 'special')
        .unsupported()
        .pure()
        .gap('squarewave.sym', 'SquareWave[x]', { expected: 'SquareWave[x]', notes: 'echo; no numeric samples' })
        .done(),
];
