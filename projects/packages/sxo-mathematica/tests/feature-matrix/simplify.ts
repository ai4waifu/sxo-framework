import { feature } from '@sxo/harness';

export const simplifyFeatures = [
    feature('Simplify', 'simplify').supported().pure().eval('simplify.trig', 'Simplify[Sin[x]^2 + Cos[x]^2]', '1').done(),
    feature('FullSimplify', 'simplify')
        .unsupported()
        .pure()
        .gap('fullsimplify.trig', 'FullSimplify[Sin[x]^2 + Cos[x]^2]', { expected: '1' })
        .done(),
    feature('TrigExpand', 'simplify')
        .unsupported()
        .pure()
        .gap('trigexpand.sin2', 'TrigExpand[Sin[2*x]]', { expected: '2*Cos[x]*Sin[x]' })
        .done(),
    feature('TrigReduce', 'simplify')
        .unsupported()
        .pure()
        .gap('trigreduce.sin2', 'TrigReduce[Sin[x]^2]', { expected: '(1 - Cos[2*x])/2' })
        .done(),
    feature('PowerExpand', 'simplify').unsupported().pure().gap('powerexpand.sqrt', 'PowerExpand[Sqrt[x^2]]', { expected: 'x' }).done(),
    feature('Assuming', 'simplify')
        .unsupported('Assuming retained but body not simplified under assumption')
        .pure()
        .gap('assuming.sqrt', 'Assuming[x > 0, Simplify[Sqrt[x^2]]]', { expected: 'x' })
        .done(),
    feature('Refine', 'simplify').unsupported().pure().gap('refine.sqrt', 'Refine[Sqrt[x^2], x > 0]', { expected: 'x' }).done(),
];
