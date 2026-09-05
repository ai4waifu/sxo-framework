import { feature } from '@sxo/harness';

export const sessionFeatures = [
    feature('CompoundExpression', 'session')
        .partial('returns last value; no persistent Set side effects')
        .pure()
        .eval('compound.last', '1 + 2; 3 * 4', '12')
        .done(),
    feature('Set', 'session')
        .supported()
        .stateful()
        .notes('compound Set binds in one evaluate string')
        .eval('set.compound', 'x = 5; x + 1', '6')
        .eval('set.persist', 'x = 5', '5', { notes: 'follow-up x+1 on same Session → 6 (napi session test)' })
        .done(),
    feature('SetDelayed', 'session')
        .partial('symbol := returns Null; patterned f[x_]:= still pending')
        .stateful()
        .eval('setdelayed.symbol', 'a := 1 + 1', 'Null')
        .gap('setdelayed.def', 'f[x_] := x^2; f[3]', { expected: '9', notes: 'patterned SetDelayed still pending' })
        .done(),
    feature('Module', 'session')
        .supported()
        .stateful()
        .notes('local Set bind with $n unique rename')
        .eval('module.bind', 'Module[{x = 1}, x + 1]', '2')
        .done(),
    feature('With', 'session').supported().pure().notes('lexical local Set bind').eval('with.bind', 'With[{x = 1}, x + 1]', '2').done(),
    feature('Block', 'session')
        .supported()
        .stateful()
        .notes('local Set bind for this slice')
        .eval('block.bind', 'Block[{x = 1}, x + 1]', '2')
        .done(),
    feature('Clear', 'session')
        .unsupported('SILENT WRONG: Clear[x] returns x')
        .stateful()
        .gap('clear.strip', 'Clear[x]', { expected: 'Null', notes: 'currently returns x' })
        .done(),
    feature('UpSet', 'session').planned().stateful().gap('upset.basic', 'UpSet[f[x], 1]', { expected: '1' }).done(),
    feature('TagSet', 'session').planned('oak error on x/:f[x]=1').stateful().gap('tagset.basic', 'x /: f[x] = 1', { expected: '1' }).done(),
    feature('Unset', 'session').planned('oak error on a=.=').stateful().gap('unset.basic', 'a =.', { expected: 'Null' }).done(),
    feature('DynamicModule', 'session')
        .unsupported('SILENT WRONG: DynamicModule[{x=1},x] → DynamicModule[{1}, x]')
        .stateful()
        .gap('dynamicmodule.bind', 'DynamicModule[{x = 1}, x]', { expected: '1', notes: 'currently DynamicModule[{1}, x]' })
        .done(),
    feature('PrependTo', 'session')
        .unsupported('SILENT WRONG: PrependTo[x={1},0] → PrependTo[{1},0] (loses symbol)')
        .stateful()
        .gap('prependto.x', 'x = {1}; PrependTo[x, 0]; x', { expected: '{0, 1}' })
        .done(),
    feature('CompoundExpressionSet', 'session')
        .unsupported('SILENT WRONG: CompoundExpression[a=1,a] → a (no binding)')
        .stateful()
        .gap('compound.set', 'CompoundExpression[a = 1, a]', { expected: '1', notes: 'currently returns a' })
        .done(),
    feature('ReapSow', 'session')
        .unsupported('SILENT WRONG: Reap[Sow[1];Sow[2]] → Reap[Sow[2]] (CompoundExpression last-only + no Reap collect)')
        .stateful()
        .gap('reap.basic', 'Reap[Sow[1]]', { expected: '{1, {{1}}}' })
        .gap('reap.two', 'Reap[Sow[1]; Sow[2]]', { expected: '{2, {{1, 2}}}', notes: 'currently Reap[Sow[2]]' })
        .done(),
];
