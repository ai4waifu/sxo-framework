import { feature } from '@sxo/harness';

export const complexFeatures = [
    feature('Re', 'complex').partial('exact `Re` on tested machine-complex atoms only').pure().eval('re.i', 'Re[I]', '0').done(),
    feature('Im', 'complex').partial('exact `Im` on tested machine-complex atoms only').pure().eval('im.i', 'Im[I]', '1').done(),
    feature('Conjugate', 'complex')
        .partial('exact `Conjugate` on tested machine-complex atoms only')
        .pure()
        .eval('conj.i', 'Conjugate[I]', '-I')
        .done(),
    feature('Arg', 'complex')
        .partial('exact `Arg` on principal-axis machine-complex atoms and exact real scalars only')
        .pure()
        .eval('arg.i', 'Arg[I]', 'Pi/2')
        .eval('arg.1', 'Arg[1]', '0')
        .eval('arg.neg1', 'Arg[-1]', 'Pi')
        .eval('arg.negi', 'Arg[-I]', '-Pi/2')
        .done(),
    feature('ComplexMul', 'complex')
        .partial('exact machine-complex multiply on tested `1±I` conjugate pair only')
        .pure()
        .eval('complexmul.conj', '(1 + I)*(1 - I)', '2')
        .done(),
    feature('ComplexScalarTimes', 'complex')
        .partial('exact integer scalar times imaginary unit on tested atoms only')
        .pure()
        .eval('neg.i', '(-1)*I', '-I')
        .eval('two.i', '2*I', '2*I')
        .done(),
];
