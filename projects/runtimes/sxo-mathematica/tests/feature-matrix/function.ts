import { feature } from '@sxo/harness';

export const functionFeatures = [
    feature('Function', 'function')
        .supported()
        .pure()
        .notes('Slot pure function and named Function application')
        .eval('function.slot', '(#^2)&[4]', '16')
        .eval('function.named', 'Function[x, x^2][3]', '9')
        .done(),
    feature('Nest', 'function').unsupported().pure().gap('nest.inc', 'Nest[# + 1 &, 0, 3]', { expected: '3' }).done(),
    feature('Fold', 'function').unsupported().pure().gap('fold.plus', 'Fold[Plus, 0, {1, 2, 3}]', { expected: '6' }).done(),
    feature('Thread', 'function').unsupported().pure().gap('thread.rule', 'Thread[{a, b} -> {1, 2}]', { expected: '{a -> 1, b -> 2}' }).done(),
    feature('Through', 'function')
        .unsupported('SILENT WRONG parse reshape: Through[{Sin,Cos}[0]] → Through[{Sin, Cos}, 0]')
        .pure()
        .gap('through.sincos', 'Through[{Sin, Cos}[0]]', { expected: '{0, 1}' })
        .done(),
    feature('Composition', 'function')
        .unsupported()
        .pure()
        .gap('composition.sincos', 'Composition[Sin, Cos][0]', { expected: 'Sin[1]' })
        .done(),
    feature('RightComposition', 'function')
        .unsupported('SILENT WRONG: f/*g → g (operator stripped to last symbol)')
        .pure()
        .gap('rightcomp.fg', 'f/*g', { expected: 'f/*g', notes: 'currently returns g' })
        .done(),
    feature('CompositionOp', 'function')
        .unsupported('SILENT WRONG: g@*f → f')
        .pure()
        .gap('compop.gf', 'g@*f', { expected: 'g@*f', notes: 'currently returns f' })
        .done(),
    feature('Construct', 'function').unsupported().pure().gap('construct.f12', 'Construct[f, 1, 2]', { expected: 'f[1, 2]' }).done(),
    feature('ApplySequence', 'function')
        .partial('Apply[Sequence,{1,2}] stays Sequence[1,2]; infix Sequence@@ may stay Apply')
        .pure()
        .gap('apply.sequence_head', 'Apply[Sequence, {1, 2}]', { expected: '{1, 2}', notes: 'currently Sequence[1, 2]' })
        .gap('apply.sequence_infix', 'Sequence @@ {1, 2}', { expected: '{1, 2}' })
        .done(),
];
