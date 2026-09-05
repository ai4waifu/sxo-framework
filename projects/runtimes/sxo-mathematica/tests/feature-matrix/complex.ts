import { feature } from '@sxo/harness';

export const complexFeatures = [
    feature('Re', 'complex').unsupported().pure().gap('re.i', 'Re[I]', { expected: '0' }).done(),
    feature('Im', 'complex').unsupported().pure().gap('im.i', 'Im[I]', { expected: '1' }).done(),
    feature('Conjugate', 'complex').unsupported().pure().gap('conj.i', 'Conjugate[I]', { expected: '-I' }).done(),
    feature('Arg', 'complex').unsupported().pure().gap('arg.i', 'Arg[I]', { expected: 'Pi/2' }).done(),
    feature('ComplexMul', 'complex')
        .unsupported('(1+I)*(1-I) → 1+-(I^2) not folded to 2')
        .pure()
        .gap('complexmul.conj', '(1 + I)*(1 - I)', { expected: '2', notes: 'currently 1 + -(I^2)' })
        .done(),
];
