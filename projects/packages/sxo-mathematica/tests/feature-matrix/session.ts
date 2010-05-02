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
        .supported()
        .stateful()
        .notes('symbol := stores residual and evaluates on use; patterned f[x_]:= dispatches via TermPattern')
        .eval('setdelayed.symbol', 'a := 1 + 1', 'Null')
        .eval('setdelayed.use', 'a := 1 + 1; a', '2')
        .eval('setdelayed.def', 'f[x_] := x^2; f[3]', '9')
        .done(),
    feature('Module', 'session')
        .supported()
        .stateful()
        .notes('local Set bind with $n unique rename; bare Module[{x},x] ignores session Own (dialect regression)')
        .eval('module.bind', 'Module[{x = 1}, x + 1]', '2')
        .done(),
    feature('With', 'session')
        .supported()
        .pure()
        .notes('simultaneous lexical RHS substitution; With[{x=1,y=x},y] keeps outer x')
        .eval('with.bind', 'With[{x = 1}, x + 1]', '2')
        .eval('with.simultaneous', 'With[{x = 1, y = x}, y]', 'x')
        .eval('with.outer', 'x = 5; With[{x = 1, y = x}, y]', '5')
        .done(),
    feature('Block', 'session')
        .supported()
        .stateful()
        .notes('dynamic shadow under DynamicScope; bare Block[{x},x] clears Own and restores')
        .eval('block.bind', 'Block[{x = 1}, x + 1]', '2')
        .eval('block.clear', 'x = 5; Block[{x}, x]', 'x')
        .eval('block.restore', 'x = 5; Block[{x = 1}, x]; x', '5')
        .done(),
    feature('Clear', 'session')
        .supported()
        .stateful()
        .notes('ClearDefinition → Null; strips session Own bindings')
        .eval('clear.strip', 'Clear[x]', 'Null')
        .eval('clear.unbind', 'x = 5; Clear[x]; x', 'x')
        .done(),
    feature('UpSet', 'session').planned().stateful().gap('upset.basic', 'UpSet[f[x], 1]', { expected: '1' }).done(),
    feature('TagSet', 'session').planned('oak error on x/:f[x]=1').stateful().gap('tagset.basic', 'x /: f[x] = 1', { expected: '1' }).done(),
    feature('Unset', 'session').planned('oak error on a=.=').stateful().gap('unset.basic', 'a =.', { expected: 'Null' }).done(),
    feature('DynamicModule', 'session')
        .unsupported('unevaluated DynamicModule[{Set[x,1]}, x] (no frontend Dynamic eval)')
        .stateful()
        .gap('dynamicmodule.bind', 'DynamicModule[{x = 1}, x]', {
            expected: '1',
            notes: 'stays DynamicModule[{Set[x, 1]}, x]; no longer collapses binder to {1}',
        })
        .done(),
    feature('PrependTo', 'session')
        .unsupported('PrependTo does not mutate session list binding yet')
        .stateful()
        .gap('prependto.x', 'x = {1}; PrependTo[x, 0]; x', {
            expected: '{0, 1}',
            notes: 'x remains {1}; not the old PrependTo[{1},0] symbol-loss shape',
        })
        .done(),
    feature('CompoundExpressionSet', 'session')
        .supported()
        .stateful()
        .notes('head-form CompoundExpression with Set binds like semicolon compound')
        .eval('compound.set', 'CompoundExpression[a = 1, a]', '1')
        .done(),
    feature('ReapSow', 'session')
        .unsupported('unevaluated Reap/Sow (no collect journal)')
        .stateful()
        .gap('reap.basic', 'Reap[Sow[1]]', {
            expected: '{1, {{1}}}',
            notes: 'stays Reap[Sow[1]]',
        })
        .gap('reap.two', 'Reap[Sow[1]; Sow[2]]', {
            expected: '{2, {{1, 2}}}',
            notes: 'stays Reap[CompoundExpression[Sow[1], Sow[2]]] (compound preserved inside Reap)',
        })
        .done(),
];
