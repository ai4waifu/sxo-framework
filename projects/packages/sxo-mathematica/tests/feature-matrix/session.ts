import { feature } from '@sxo/harness';

export const sessionFeatures = [
    feature('CompoundExpression', 'session')
        .partial('returns last value; no persistent Set side effects')
        .pure()
        .eval('compound.last', '1 + 2; 3 * 4', '12')
        .done(),
    feature('Set', 'session')
        .partial('compound `Set` in one evaluate string on tested forms')
        .stateful()
        .eval('set.compound', 'x = 5; x + 1', '6')
        .eval('set.persist', 'x = 5', '5', { notes: 'follow-up x+1 on same Session → 6 (napi session test)' })
        .done(),
    feature('SetDelayed', 'session')
        .partial('`:=` and tested pattern `f[x_]:=` in one Session only')
        .stateful()
        .eval('setdelayed.symbol', 'a := 1 + 1', 'Null')
        .eval('setdelayed.use', 'a := 1 + 1; a', '2')
        .eval('setdelayed.def', 'f[x_] := x^2; f[3]', '9')
        .done(),
    feature('Module', 'session')
        .partial('local `Set` bind with `$n` rename on tested forms')
        .stateful()
        .eval('module.bind', 'Module[{x = 1}, x + 1]', '2')
        .done(),
    feature('With', 'session')
        .partial('simultaneous lexical substitution on tested forms')
        .pure()
        .eval('with.bind', 'With[{x = 1}, x + 1]', '2')
        .eval('with.simultaneous', 'With[{x = 1, y = x}, y]', 'x')
        .eval('with.outer', 'x = 5; With[{x = 1, y = x}, y]', '5')
        .done(),
    feature('Block', 'session')
        .partial('dynamic shadow under `DynamicScope` on tested forms')
        .stateful()
        .eval('block.bind', 'Block[{x = 1}, x + 1]', '2')
        .eval('block.clear', 'x = 5; Block[{x}, x]', 'x', {
            notes: 'compound + autoSimplify keep free x (result transform CaptureAsTerm on Simplify)',
        })
        .eval('block.restore', 'x = 5; Block[{x = 1}, x]; x', '5')
        .done(),
    feature('Clear', 'session')
        .partial('`Clear` strips session `Own` bindings on tested symbols')
        .stateful()
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
        .partial('head-form `CompoundExpression` with `Set` on tested forms')
        .stateful()
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
