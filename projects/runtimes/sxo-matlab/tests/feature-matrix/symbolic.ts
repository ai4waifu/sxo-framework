import { feature } from '@sxo/harness';

export const symbolicFeatures = [
    feature('sym', 'symbolic').unsupported().pure().gap('sym.x', "sym('x')", { expected: 'x' }).done(),
    feature('vpa', 'symbolic').unsupported().pure().gap('vpa.pi', 'vpa(pi, 10)', { expected: '3.141592654' }).done(),
    feature('syms', 'symbolic')
        .unsupported('SILENT WRONG: syms x → x (declaration stripped like global)')
        .stateful()
        .gap('syms.strip', 'syms x', { expected: '...', notes: 'currently returns x' })
        .done(),
    feature('expand', 'symbolic')
        .unsupported('args already wrongly powered: expand((x+1)^2) sees expand(1+x^2)')
        .pure()
        .gap('expand.binomsq', 'expand((x + 1)^2)', { expected: 'x^2 + 2*x + 1' })
        .done(),
    feature('limit', 'symbolic').unsupported().pure().gap('limit.sinc', 'limit(sin(x)/x, x, 0)', { expected: '1' }).done(),
    feature('dsolve', 'symbolic').planned().pure().gap('dsolve.exp', 'dsolve(diff(y)==y)', { expected: 'C1*exp(t)' }).done(),
    feature('simplify_trig', 'symbolic')
        .supported()
        .pure()
        .notes('same identity as simplify entry; explicit symbolic toolbox spelling')
        .eval('simplify_trig.pythag', 'simplify(sin(x)^2 + cos(x)^2)', '1')
        .done(),
    feature('fourier_sym', 'symbolic')
        .unsupported('SILENT WRONG: fourier(exp(-x^2)) → fourier(exp(x^2)) sign flip')
        .pure()
        .gap('fourier.gauss_sign', 'fourier(exp(-x^2))', { expected: '...', notes: 'currently fourier(exp(x^2))' })
        .done(),
    feature('latex_sym', 'symbolic').unsupported().pure().gap('latex.x2', "latex(sym('x^2'))", { expected: '...' }).done(),
];
