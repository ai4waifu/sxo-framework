import { feature } from '@sxo/harness';

export const specialFeatures = [
    feature('Gamma', 'special').unsupported().pure().gap('gamma.5', 'Gamma[5]', { expected: '24' }).done(),
    feature('Zeta', 'special').unsupported().pure().gap('zeta.2', 'Zeta[2]', { expected: 'Pi^2/6' }).done(),
    feature('Erf', 'special').unsupported().pure().gap('erf.0', 'Erf[0]', { expected: '0' }).done(),
    feature('UnitStep', 'special')
        .unsupported('UnitStep[1] unevaluated; D[UnitStep[x],x] host stack-overflow crash — keep as gap only')
        .pure()
        .gap('unitstep.1', 'UnitStep[1]', { expected: '1' })
        .gap('unitstep.deriv_crash', 'D[UnitStep[x], x]', {
            expected: 'DiracDelta[x]',
            notes: 'host crash (stack overflow) observed — do not promote to eval',
        })
        .done(),
    feature('HeavisideTheta', 'special').unsupported().pure().gap('heaviside.1', 'HeavisideTheta[1]', { expected: '1' }).done(),
    feature('BesselJ', 'special').unsupported().pure().gap('besselj.10', 'BesselJ[1, 0]', { expected: '0' }).done(),
    feature('LegendreP', 'special').unsupported().pure().gap('legendrep.2', 'LegendreP[2, x]', { expected: '(-1 + 3*x^2)/2' }).done(),
    feature('GammaHalf', 'special').unsupported().pure().gap('gamma.half', 'Gamma[1/2]', { expected: 'Sqrt[Pi]' }).done(),
    feature('ZetaZero', 'special').unsupported().pure().gap('zeta.0', 'Zeta[0]', { expected: '-1/2' }).done(),
    feature('Sinc', 'special')
        .unsupported()
        .pure()
        .gap('sinc.0', 'Sinc[0]', { expected: '1' })
        .gap('sinc.pi', 'Sinc[Pi]', { expected: '0' })
        .done(),
    feature('FresnelC', 'special').unsupported().pure().gap('fresnelc.inf', 'FresnelC[Infinity]', { expected: '1/2' }).done(),
    feature('InverseErf', 'special').unsupported().pure().gap('inverseerf.0', 'InverseErf[0]', { expected: '0' }).done(),
    feature('SquareWave', 'special')
        .unsupported()
        .pure()
        .gap('squarewave.sym', 'SquareWave[x]', { expected: 'SquareWave[x]', notes: 'echo; no numeric samples' })
        .done(),
];
