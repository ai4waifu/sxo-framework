import { feature } from '@sxo/harness';

export const calculusFeatures = [
    feature('D', 'calculus')
        .partial('poly/trig/chain and D[x*y,x] OK; bare juxtaposition D[x y,x]→D[x,y,x]; D[f[x],x]/compose crash host')
        .pure()
        .eval('d.poly', 'D[x^3, x]', '3*x^2')
        .eval('d.sin2', 'D[Sin[x], {x, 2}]', '-Sin[x]')
        .eval('d.chain', 'D[Sin[x^2], x]', '2*x*Cos[x^2]')
        .eval('d.juxtapose', 'D[x*y, x]', 'y', { notes: 'bare D[x y,x] currently D[x, y, x]' })
        .gap('d.bare_juxtapose', 'D[x y, x]', { expected: 'y', notes: 'SILENT WRONG: currently D[x, y, x]' })
        .gap('d.symbolic_head', 'D[f[x], x]', {
            expected: "f'[x]",
            notes: 'host crash (stack overflow) observed — keep as gap, do not promote to eval',
        })
        .gap('d.order0_crash', 'D[f[x], {x, 0}]', { expected: 'f[x]', notes: 'host crash (stack overflow) — keep as gap only' })
        .gap('d.compose_crash', 'D[f[g[x]], x]', { expected: "f'[g[x]]*g'[x]", notes: 'host crash (stack overflow) — keep as gap only' })
        .done(),
    feature('Integrate', 'calculus')
        .partial('indefinite poly/sin ok; definite Sin to Pi exact; juxtaposition/Exp[-x^2] gaps remain')
        .pure()
        .eval('integrate.poly', 'Integrate[x^2, x]', '1/3*x^3')
        .eval('integrate.sin', 'Integrate[Sin[x], x]', '-Cos[x]')
        .eval('integrate.definite_sin', 'Integrate[Sin[x], {x, 0, Pi}]', '2')
        .gap('integrate.parts_juxtapose', 'Integrate[x*Sin[x], x]', { expected: '-x*Cos[x] + Sin[x]' })
        .gap('integrate.log', 'Integrate[1/x, x]', { expected: 'Log[x]', notes: 'currently unevaluated Integrate[x^-1, x]' })
        .gap('integrate.gauss_sign', 'Integrate[Exp[-x^2], {x, -Infinity, Infinity}]', {
            expected: 'Sqrt[Pi]',
            notes: 'currently Integrate[Exp[x^2], …]',
        })
        .done(),
    feature('Limit', 'calculus')
        .partial('sinc and Infinity OK; (1+x)^(1/x) still gap')
        .pure()
        .eval('limit.sinc', 'Limit[Sin[x]/x, x -> 0]', '1')
        .gap('limit.exp', 'Limit[(1 + x)^(1/x), x -> 0]', { expected: 'E', notes: 'currently 1^0^-1' })
        .eval('limit.inf', 'Limit[1/x, x -> Infinity]', '0')
        .done(),
    feature('Series', 'calculus')
        .partial('Exp order-2 OK as float 0.5; order-3 drops /6 (…+x^3 not …+x^3/6); Sin series wrong; Normal wrapper unevaluated')
        .pure()
        .eval('series.exp', 'Series[Exp[x], {x, 0, 2}]', '1 + x + 0.5*x^2')
        .gap('series.exp3', 'Series[Exp[x], {x, 0, 3}]', { expected: '1 + x + x^2/2 + x^3/6', notes: 'currently 1 + x + 0.5*x^2 + x^3' })
        .gap('series.sin', 'Series[Sin[x], {x, 0, 3}]', { expected: 'x - x^3/6', notes: 'currently x + -(x^3)' })
        .done(),
    feature('LaplaceTransform', 'calculus')
        .unsupported('nested/garbage ROCUnknown re-wrapping')
        .pure()
        .gap('laplace.exp', 'LaplaceTransform[Exp[-a*t], t, s]', { expected: '1/(a + s)' })
        .done(),
    feature('FourierTransform', 'calculus')
        .unsupported('SILENT WRONG: Exp[-x^2] becomes Exp[x^2] in residual form')
        .pure()
        .gap('fourier.gauss', 'FourierTransform[Exp[-x^2], x, k]', { expected: 'Sqrt[Pi]*Exp[-k^2/4]' })
        .done(),
    feature('Dt', 'calculus').unsupported().pure().gap('dt.x2', 'Dt[x^2]', { expected: '2*x*Dt[x]' }).done(),
    feature('Derivative', 'calculus').unsupported().pure().gap('derivative.sin', 'Derivative[1][Sin][x]', { expected: 'Cos[x]' }).done(),
    feature('Fourier', 'calculus').unsupported().pure().gap('fourier.vec', 'Fourier[{1, 2, 3, 4}]', { expected: '...' }).done(),
    feature('InverseFourier', 'calculus')
        .unsupported()
        .pure()
        .gap('inversefourier.impulse', 'InverseFourier[{1, 0, 0, 0}]', { expected: '...' })
        .done(),
    feature('Residue', 'calculus')
        .partial(
            'some simple poles OK (1/z, Exp[z]/z); Residue[1/(z-1),{z,1}] → 0 SILENT WRONG (expect 1); partial fractions often unevaluated',
        )
        .pure()
        .eval('residue.1_z', 'Residue[1/z, {z, 0}]', '1')
        .eval('residue.exp_z', 'Residue[Exp[z]/z, {z, 0}]', '1')
        .gap('residue.shift', 'Residue[1/(z - 1), {z, 1}]', { expected: '1', notes: 'currently returns 0' })
        .done(),
    feature('InverseLaplaceTransform', 'calculus')
        .unsupported()
        .pure()
        .gap('ilaplace.exp', 'InverseLaplaceTransform[1/(s + a), s, t]', { expected: 'Exp[-a*t]' })
        .done(),
    feature('DAbs', 'calculus')
        .partial('D[Abs[x],x] → Abs[x]/x form (x^-1*Abs[x]); acceptable rewrite, not Sign[x]')
        .pure()
        .eval('dabs.x', 'D[Abs[x], x]', 'x^-1*Abs[x]')
        .done(),
    feature('Curl', 'calculus')
        .unsupported('SILENT WRONG: Curl[{-y,x},{x,y}] → {} (expect 2)')
        .pure()
        .gap('curl.2d', 'Curl[{-y, x}, {x, y}]', { expected: '2', notes: 'currently {}' })
        .done(),
    feature('Grad', 'calculus').unsupported().pure().gap('grad.xy', 'Grad[x*y, {x, y}]', { expected: '{y, x}' }).done(),
    feature('Div', 'calculus').unsupported().pure().gap('div.xy', 'Div[{x, y}, {x, y}]', { expected: '2' }).done(),
    feature('ZTransform', 'calculus')
        .unsupported('SILENT WRONG: nested ZTransform[…, ROCUnknown] re-wrapping (same family as LaplaceTransform)')
        .pure()
        .gap('ztransform.n', 'ZTransform[n, n, z]', {
            expected: 'z/(-1 + z)^2',
            notes: 'currently ZTransform[ZTransform[n, n, z], {n, z}, ROCUnknown]',
        })
        .done(),
    feature('InverseZTransform', 'calculus')
        .unsupported()
        .pure()
        .gap('iztrans.step', 'InverseZTransform[z/(z - 1), z, n]', { expected: '1' })
        .done(),
    feature('FourierSequenceTransform', 'calculus')
        .unsupported()
        .pure()
        .gap('fst.a_n', 'FourierSequenceTransform[a^n, n, w]', { expected: '...' })
        .done(),
    feature('GeneratingFunction', 'calculus')
        .unsupported('Factorial head appears; no closed form')
        .pure()
        .gap('genfun.factorial', 'GeneratingFunction[n!, n, x]', {
            expected: '1/(1 - x)',
            notes: 'actually egf for n!; ogf different — record unevaluated',
        })
        .done(),
];
