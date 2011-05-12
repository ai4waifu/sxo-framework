import { feature } from '@sxo/harness';

export const complexFeatures = [
    feature('real', 'complex').partial('exact `real` on tested `1i` machine-complex atoms only').pure().eval('real.1i', 'real(1i)', '0').done(),
    feature('imag', 'complex').partial('exact `imag` on tested `1i` machine-complex atoms only').pure().eval('imag.1i', 'imag(1i)', '1').done(),
    feature('conj', 'complex')
        .partial('exact `conj` on tested `1i` machine-complex atoms only')
        .pure()
        .eval('conj.1i', 'conj(1i)', '-1i')
        .done(),
    feature('angle', 'complex')
        .partial('exact `angle` on principal-axis machine-complex atoms and exact real scalars only')
        .pure()
        .eval('angle.1i', 'angle(1i)', 'pi/2')
        .eval('angle.1', 'angle(1)', '0')
        .eval('angle.neg1', 'angle(-1)', 'pi')
        .eval('angle.neg1i', 'angle(-1i)', '-pi/2')
        .done(),
    feature('complex_mul', 'complex')
        .partial('exact machine-complex multiply on tested `1±1i` conjugate pair only')
        .pure()
        .eval('complexmul.conj', '(1 + 1i)*(1 - 1i)', '2')
        .done(),
    feature('complex_scalar_times', 'complex')
        .partial('exact integer scalar times imaginary unit on tested atoms only')
        .pure()
        .eval('neg.1i', '(-1)*1i', '-1i')
        .eval('two.1i', '2*1i', '2i')
        .done(),
];
