import { feature } from '@sxo/harness';

export const complexFeatures = [
    feature('Re', 'complex').partial('exact `Re` on tested machine-complex atoms only').pure().eval('re.i', 'Re[I]', '0').done(),
    feature('Im', 'complex').partial('exact `Im` on tested machine-complex atoms only').pure().eval('im.i', 'Im[I]', '1').done(),
    feature('Conjugate', 'complex')
        .partial('exact `Conjugate` on tested machine-complex atoms only')
        .pure()
        .eval('conj.i', 'Conjugate[I]', '-I')
        .done(),
    feature('Arg', 'complex').unsupported().pure().gap('arg.i', 'Arg[I]', { expected: 'Pi/2' }).done(),
    feature('ComplexMul', 'complex')
        .unsupported('(1+I)*(1-I) → 1+-(I^2) not folded to 2')
        .pure()
        .gap('complexmul.conj', '(1 + I)*(1 - I)', { expected: '2', notes: 'currently 1 + -(I^2)' })
        .done(),
];
