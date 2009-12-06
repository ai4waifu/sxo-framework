import { feature, matrix } from '@sxo/harness';

/**
 * Mathematica dialect capability matrix.
 *
 * Status is honest against current SXO + Athena behavior.
 * `gap` cases track contract items for upstream handoff.
 * Authored with `@sxo/harness` builders (not raw object literals).
 */
export const featureMatrix = matrix(
    feature('Plus', 'arithmetic').supported().pure().eval('plus.basic', '1 + 2 * 3', '7').eval('plus.nary', 'Plus[1, 2, 3]', '6').done(),
    feature('Times', 'arithmetic').supported().pure().eval('times.nary', 'Times[2, 3, 4]', '24').done(),
    feature('Power', 'arithmetic').supported().pure().eval('power.pow1', 'Power[x, 1]', 'x').eval('power.square', '2^3', '8').done(),
    feature('Subtract', 'arithmetic').supported().pure().eval('subtract.basic', 'Subtract[5, 2]', '3').done(),
    feature('Divide', 'arithmetic')
        .supported()
        .pure()
        .eval('divide.basic', 'Divide[6, 2]', '3')
        .eval('divide.rational', '1/3 + 1/3 + 1/3', '1')
        .done(),
    feature('Factorial', 'arithmetic').supported().pure().eval('factorial.5', '5!', '120').done(),
    feature('Sqrt', 'arithmetic').supported().pure().eval('sqrt.4', 'Sqrt[4]', '2').done(),
    feature('Abs', 'arithmetic').supported().pure().eval('abs.neg', 'Abs[-3]', '3').done(),
    feature('Equal', 'comparison').supported().pure().eval('equal.true', '2 == 2', 'True').done(),
    feature('Unequal', 'comparison').supported().pure().eval('unequal.true', '2 != 3', 'True').done(),
    feature('Less', 'comparison').supported().pure().eval('less.infix', '2 < 3', 'True').eval('less.head', 'Less[2, 3]', 'True').done(),
    feature('Greater', 'comparison').supported().pure().eval('greater.infix', '3 > 2', 'True').done(),
    feature('LessEqual', 'comparison')
        .partial('binary <= / LessEqual returns typed Boolean; n-ary LessEqual[1,2,3] unevaluated')
        .pure()
        .eval('le.infix', '2 <= 3', 'True')
        .eval('le.eq', 'LessEqual[1, 1]', 'True')
        .gap('le.chain', 'LessEqual[1, 2, 3]', { expected: 'True' })
        .done(),
    feature('GreaterEqual', 'comparison')
        .partial('binary >= / GreaterEqual returns typed Boolean; Inequality chain unevaluated')
        .pure()
        .eval('ge.infix', '3 >= 2', 'True')
        .eval('ge.eq', 'GreaterEqual[2, 2]', 'True')
        .gap('ge.inequality', 'Inequality[1, Less, 2, Less, 3]', { expected: 'True' })
        .done(),
    feature('And', 'logic')
        .supported()
        .pure()
        .notes('typed Boolean atoms; And over Equal and True/False')
        .eval('and.equal', 'And[1 == 1, 2 == 2]', 'True')
        .eval('and.bool_atoms', 'And[True, False]', 'False')
        .done(),
    feature('Or', 'logic')
        .supported()
        .pure()
        .eval('or.equal', 'Or[1 == 2, 2 == 2]', 'True')
        .eval('or.bool_atoms', 'Or[False, True]', 'True')
        .done(),
    feature('Not', 'logic').supported().pure().eval('not.equal', 'Not[1 == 2]', 'True').eval('not.true', 'Not[True]', 'False').done(),
    feature('If', 'logic')
        .supported()
        .pure()
        .notes('short-circuit; non-boolean condition is structured diagnostic at Athena')
        .eval('if.true', 'If[1 == 1, 7, 8]', '7')
        .done(),
    feature('Which', 'logic').supported().pure().eval('which.basic', 'Which[False, 1, True, 2]', '2').done(),
    feature('List', 'list')
        .supported()
        .pure()
        .eval('list.literal', '{1, 2, 3}', '{1, 2, 3}')
        .roundtrip('list.roundtrip', '{1, 2}', '{1, 2}')
        .done(),
    feature('Part', 'list')
        .supported()
        .pure()
        .notes('1-based Part; [[0]] returns head List')
        .eval('part.infix', '{1, 2, 3}[[2]]', '2')
        .eval('part.head', 'Part[{1, 2, 3}, 1]', '1')
        .eval('part.zero', '{1, 2, 3}[[0]]', 'List')
        .done(),
    feature('Range', 'list').supported().pure().notes('Range[n] integer sequence').eval('range.3', 'Range[3]', '{1, 2, 3}').done(),
    feature('Map', 'list')
        .partial('Slot Map OK; Map[Sin,{0,1}] currently numericizes Sin[1]')
        .pure()
        .gap('map.sin', 'Map[Sin, {0, 1}]', { expected: '{0, Sin[1]}', notes: 'currently {0, 0.8414709848078965}' })
        .eval('map.slot', 'Map[#^2 &, {1, 2, 3}]', '{1, 4, 9}')
        .done(),
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
    feature('Function', 'function')
        .supported()
        .pure()
        .notes('Slot pure function and named Function application')
        .eval('function.slot', '(#^2)&[4]', '16')
        .eval('function.named', 'Function[x, x^2][3]', '9')
        .done(),
    feature('Rule', 'rule')
        .partial('head ReplaceAll[expr, lhs->rhs] works for simple symbols; RuleDelayed / infix /. fragile')
        .pure()
        .eval('rule.replaceall_pow', 'ReplaceAll[x^2, x -> 3]', '9')
        .gap('rule.replaceall_named', 'ReplaceAll[x^2, Rule[x, 3]]', { expected: '9', notes: 'Rule[…] may flatten args' })
        .done(),
    feature('RuleDelayed', 'rule').planned().pure().gap('ruledelayed.basic', 'x /. x :> 1 + 1', { expected: '2' }).done(),
    feature('Replace', 'rule').unsupported().pure().gap('replace.basic', 'Replace[a, a -> 1]', { expected: '1' }).done(),
    feature('ReplaceAll', 'rule')
        .partial(
            'head form works for symbol and list-of-rules; infix /. often oak error; Hold args evaluate first so ReplaceAll[Hold[1+1],1->2] → Hold[2] (cascades Hold silent wrong)',
        )
        .pure()
        .eval('replaceall.head', 'ReplaceAll[x, x -> 3]', '3')
        .eval('replaceall.list_rules', 'ReplaceAll[{a, b}, {a -> 1, b -> 2}]', '{1, 2}')
        .gap('replaceall.infix', 'x^2 /. x -> 3', { expected: '9' })
        .gap('replaceall.into_hold', 'ReplaceAll[Hold[1 + 1], 1 -> 2]', {
            expected: 'Hold[2 + 2]',
            notes: 'currently Hold[2] after Hold evaluates Plus',
        })
        .done(),
    feature('Hold', 'hold').supported().unevaluated().notes('HoldAll args preserved').eval('hold.plus', 'Hold[1 + 1]', 'Hold[1 + 1]').done(),
    feature('HoldForm', 'hold').supported().unevaluated().eval('holdform.plus', 'HoldForm[1 + 1]', 'HoldForm[1 + 1]').done(),
    feature('HoldAll', 'attributes').planned().unevaluated().gap('attr.holdall', 'Attributes[Hold]', { expected: '{HoldAll}' }).done(),
    feature('Listable', 'attributes').planned().pure().gap('attr.listable', 'Sin[{0, Pi}]', { expected: '{0, 0}' }).done(),
    feature('Orderless', 'attributes').planned().pure().gap('attr.orderless', 'Plus[b, a]', { expected: 'a + b' }).done(),
    feature('Sin', 'elementary').supported().pure().eval('sin.0', 'Sin[0]', '0').done(),
    feature('Cos', 'elementary').supported().pure().eval('cos.0', 'Cos[0]', '1').done(),
    feature('Tan', 'elementary').supported().pure().eval('tan.0', 'Tan[0]', '0').done(),
    feature('Exp', 'elementary').supported().pure().eval('exp.0', 'Exp[0]', '1').done(),
    feature('Log', 'elementary').supported().pure().eval('log.1', 'Log[1]', '0').done(),
    feature('Simplify', 'simplify').supported().pure().eval('simplify.trig', 'Simplify[Sin[x]^2 + Cos[x]^2]', '1').done(),
    feature('D', 'calculus')
        .partial('poly/trig/chain and D[x*y,x] OK; bare juxtaposition D[x y,x]→D[x,y,x]; D[f[x],x]/compose crash host')
        .pure()
        .eval('d.poly', 'D[x^3, x]', '3*x^2')
        .eval('d.sin2', 'D[Sin[x], {x, 2}]', '-Sin[x]')
        .eval('d.chain', 'D[Sin[x^2], x]', '2*x*Cos[x^2]')
        .eval('d.juxtapose', 'D[x*y, x]', 'y', { notes: 'bare D[x y,x] currently D[x, y, x]' })
        .gap('d.bare_juxtapose', 'D[x y, x]', { expected: 'y', notes: 'SILENT WRONG: currently D[x, y, x]' })
        .gap('d.symbolic_head', 'D[f[x], x]', {
            expected: "f'[x]",
            notes: 'host crash (stack overflow) observed — keep as gap, do not promote to eval',
        })
        .gap('d.order0_crash', 'D[f[x], {x, 0}]', { expected: 'f[x]', notes: 'host crash (stack overflow) — keep as gap only' })
        .gap('d.compose_crash', 'D[f[g[x]], x]', { expected: "f'[g[x]]*g'[x]", notes: 'host crash (stack overflow) — keep as gap only' })
        .done(),
    feature('Integrate', 'calculus')
        .partial('indefinite poly/sin ok; definite Sin to Pi exact; juxtaposition/Exp[-x^2] gaps remain')
        .pure()
        .eval('integrate.poly', 'Integrate[x^2, x]', '1/3*x^3')
        .eval('integrate.sin', 'Integrate[Sin[x], x]', '-Cos[x]')
        .eval('integrate.definite_sin', 'Integrate[Sin[x], {x, 0, Pi}]', '2')
        .gap('integrate.parts_juxtapose', 'Integrate[x*Sin[x], x]', { expected: '-x*Cos[x] + Sin[x]' })
        .gap('integrate.log', 'Integrate[1/x, x]', { expected: 'Log[x]', notes: 'currently unevaluated Integrate[x^-1, x]' })
        .gap('integrate.gauss_sign', 'Integrate[Exp[-x^2], {x, -Infinity, Infinity}]', {
            expected: 'Sqrt[Pi]',
            notes: 'currently Integrate[Exp[x^2], …]',
        })
        .done(),
    feature('Limit', 'calculus')
        .partial('sinc and Infinity OK; (1+x)^(1/x) still gap')
        .pure()
        .eval('limit.sinc', 'Limit[Sin[x]/x, x -> 0]', '1')
        .gap('limit.exp', 'Limit[(1 + x)^(1/x), x -> 0]', { expected: 'E', notes: 'currently 1^0^-1' })
        .eval('limit.inf', 'Limit[1/x, x -> Infinity]', '0')
        .done(),
    feature('Series', 'calculus')
        .partial('Exp order-2 OK as float 0.5; order-3 drops /6 (…+x^3 not …+x^3/6); Sin series wrong; Normal wrapper unevaluated')
        .pure()
        .eval('series.exp', 'Series[Exp[x], {x, 0, 2}]', '1 + x + 0.5*x^2')
        .gap('series.exp3', 'Series[Exp[x], {x, 0, 3}]', { expected: '1 + x + x^2/2 + x^3/6', notes: 'currently 1 + x + 0.5*x^2 + x^3' })
        .gap('series.sin', 'Series[Sin[x], {x, 0, 3}]', { expected: 'x - x^3/6', notes: 'currently x + -(x^3)' })
        .done(),
    feature('Solve', 'solve')
        .supported()
        .pure()
        .notes('univariate rational-root bridge; typed SolutionSet / Reduce still pending')
        .eval('solve.quad', 'Solve[x^2 == 1, x]', '{{x -> -1}, {x -> 1}}')
        .done(),
    feature('NSolve', 'solve').planned().pure().gap('nsolve.quad', 'NSolve[x^2 == 1, x]', { expected: '{{x -> -1.}, {x -> 1.}}' }).done(),
    feature('Reduce', 'solve').planned().pure().gap('reduce.basic', 'Reduce[x^2 > 0, x]', { expected: 'x < 0 || x > 0' }).done(),
    feature('FindRoot', 'solve').planned().pure().gap('findroot.basic', 'FindRoot[x^2 - 2, {x, 1}]', { expected: '{x -> 1.41421}' }).done(),
    feature('DSolve', 'solve')
        .planned()
        .pure()
        .gap('dsolve.basic', "DSolve[y'[x] == y[x], y[x], x]", { expected: '{{y[x] -> E^x C[1]}}' })
        .done(),
    feature('LinearSolve', 'linear_algebra')
        .unsupported('unevaluated echo; exact solve bridge not wired for this surface')
        .pure()
        .gap('linearsolve.2x2', 'LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]', {
            expected: '{{-4}, {9/2}}',
            notes: 'currently LinearSolve[{{1, 2}, {3, 4}}, {{5}, {6}}]',
        })
        .done(),
    feature('Det', 'linear_algebra').supported().pure().eval('det.2x2', 'Det[{{1, 2}, {3, 4}}]', '-2').done(),
    feature('Inverse', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('inverse.eye', 'Inverse[{{1, 0}, {0, 1}}]', { expected: '{{1, 0}, {0, 1}}' })
        .done(),
    feature('Transpose', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('transpose.2x2', 'Transpose[{{1, 2}, {3, 4}}]', { expected: '{{1, 3}, {2, 4}}' })
        .done(),
    feature('Dot', 'linear_algebra').unsupported().pure().gap('dot.mv', 'Dot[{{1, 2}, {3, 4}}, {1, 1}]', { expected: '{3, 7}' }).done(),
    feature('RowReduce', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('rowreduce.basic', 'RowReduce[{{1, 2}, {3, 4}}]', { expected: '{{1, 0}, {0, 1}}' })
        .done(),
    feature('Table', 'list')
        .supported()
        .pure()
        .notes('single iterator Table[i,{i,n}]')
        .eval('table.basic', 'Table[i, {i, 3}]', '{1, 2, 3}')
        .done(),
    feature('Sum', 'list')
        .supported()
        .pure()
        .notes('Sum over single iterator via Table fold')
        .eval('sum.basic', 'Sum[i, {i, 1, 10}]', '55')
        .done(),
    feature('Product', 'list')
        .supported()
        .pure()
        .notes('Product over single iterator via Table fold')
        .eval('product.basic', 'Product[i, {i, 1, 5}]', '120')
        .done(),
    feature('Length', 'list').supported().pure().eval('length.3', 'Length[{1, 2, 3}]', '3').done(),
    feature('First', 'list')
        .supported()
        .pure()
        .notes('First on non-empty List; empty → InvalidIndex at Athena')
        .eval('first.ab', 'First[{a, b}]', 'a')
        .done(),
    feature('Join', 'list').supported().pure().eval('join.basic', 'Join[{1}, {2}]', '{1, 2}').done(),
    feature('Flatten', 'list').unsupported().pure().gap('flatten.basic', 'Flatten[{{1, 2}, {3}}]', { expected: '{1, 2, 3}' }).done(),
    feature('Apply', 'list').supported().pure().eval('apply.plus', 'Apply[Plus, {1, 2, 3}]', '6').done(),
    feature('Nest', 'function').unsupported().pure().gap('nest.inc', 'Nest[# + 1 &, 0, 3]', { expected: '3' }).done(),
    feature('Fold', 'function').unsupported().pure().gap('fold.plus', 'Fold[Plus, 0, {1, 2, 3}]', { expected: '6' }).done(),
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
    feature('Blank', 'pattern')
        .supported()
        .pure()
        .notes('typed Blank[Integer] for MatchQ/Cases')
        .eval('blank.matchq', 'MatchQ[1, _Integer]', 'True')
        .eval('blank.cases', 'Cases[{1, a, 2}, _Integer]', '{1, 2}')
        .done(),
    feature('N', 'numeric')
        .unsupported()
        .pure()
        .gap('n.pi', 'N[Pi]', { expected: '3.14159' })
        .gap('n.prec', 'N[1/3, 20]', { expected: '0.33333333333333333333' })
        .done(),
    feature('GCD', 'number_theory').unsupported().pure().gap('gcd.basic', 'GCD[12, 18]', { expected: '6' }).done(),
    feature('Mod', 'number_theory').unsupported().pure().gap('mod.basic', 'Mod[10, 3]', { expected: '1' }).done(),
    feature('PrimeQ', 'number_theory').unsupported().pure().gap('primeq.7', 'PrimeQ[7]', { expected: 'True' }).done(),
    feature('Expand', 'algebra').unsupported().pure().gap('expand.bin', 'Expand[(x + 1)^2]', { expected: '1 + 2*x + x^2' }).done(),
    feature('Factor', 'algebra').unsupported().pure().gap('factor.diff', 'Factor[x^2 - 1]', { expected: '(-1 + x)*(1 + x)' }).done(),
    feature('Max', 'arithmetic').unsupported().pure().gap('max.3', 'Max[1, 3, 2]', { expected: '3' }).done(),
    feature('Floor', 'arithmetic').unsupported().pure().gap('floor.2_7', 'Floor[2.7]', { expected: '2' }).done(),
    feature('Pi', 'constant').partial('symbol retained; no numeric value without N').pure().eval('pi.symbol', 'Pi', 'Pi').done(),
    feature('I', 'constant').partial('symbol only; Re/Im/Conjugate unevaluated').pure().eval('i.symbol', 'I', 'I').done(),
    feature('Import', 'io')
        .unsupported('SILENT WRONG: Import["x.csv"] returns "x.csv" string')
        .effectful()
        .gap('import.strip', 'Import["x.csv"]', { expected: 'UnsupportedOperation', notes: 'must not strip to filename' })
        .done(),
    feature('Export', 'io')
        .unsupported('SILENT WRONG: Export["x.csv",1] returns 1')
        .effectful()
        .gap('export.strip', 'Export["x.csv", 1]', { expected: 'UnsupportedOperation' })
        .done(),
    feature('StringLength', 'string').unsupported().pure().gap('strlen.ab', 'StringLength["ab"]', { expected: '2' }).done(),
    feature('ListPlot', 'plot').unsupported().effectful().gap('listplot.basic', 'ListPlot[{{1, 1}, {2, 4}}]', { expected: '<svg' }).done(),
    feature('Plot3D', 'plot')
        .unsupported('juxtaposition x y may parse as sequence; no 3-D path')
        .effectful()
        .gap('plot3d.basic', 'Plot3D[x*y, {x, 0, 1}, {y, 0, 1}]', { expected: '<svg' })
        .done(),
    feature('Plot', 'plot')
        .partial(
            'SVG→PNG visual: curve+L-axes readable; missing tick labels, no Frame, not commercial MMA axes-at-origin. Negative domain {x,-1,1} via unary-minus fold',
        )
        .effectful()
        .plot('plot.square', 'Plot[x^2, {x, 0, 1}]', { expected: '<svg' })
        .plot('plot.sin', 'Plot[Sin[x], {x, 0, 6}]', { expected: '<svg' })
        .gap('plot.neg_domain', 'Plot[x^2, {x, -1, 1}]', { expected: '<svg', notes: 'currently not a supported 1-D plot form' })
        .done(),
    feature('ExactInteger', 'numeric').supported().pure().eval('bigint.add', '99999999999999999999 + 1', '100000000000000000000').done(),
    feature('True', 'constant').supported().pure().notes('typed Boolean atom').eval('true.atom', 'True', 'True').done(),
    feature('False', 'constant').supported().pure().notes('typed Boolean atom').eval('false.atom', 'False', 'False').done(),
    feature('Null', 'constant').supported().pure().notes('typed Null atom').eval('null.atom', 'Null', 'Null').done(),
    feature('E', 'constant').partial().pure().eval('e.symbol', 'E', 'E').done(),
    feature('ArithCanonical', 'arithmetic')
        .supported()
        .pure()
        .notes('identity folds x+0 / 1*x / x^0 / like powers')
        .eval('arith.x_plus_0', 'x + 0', 'x')
        .eval('arith.one_times_x', '1 * x', 'x')
        .eval('arith.x_pow_0', 'x^0', '1')
        .eval('arith.pow_combine', 'x^2 * x^3', 'x^5')
        .done(),
    feature('Head', 'meta')
        .unsupported('Head evaluates args first: Head[1+2] → Head[3]; Head[{1,2}] unevaluated')
        .pure()
        .gap('head.list', 'Head[{1, 2}]', { expected: 'List' })
        .gap('head.plus', 'Head[a + b]', { expected: 'Plus' })
        .done(),
    feature('Rest', 'list').unsupported().pure().gap('rest.basic', 'Rest[{1, 2, 3}]', { expected: '{2, 3}' }).done(),
    feature('Most', 'list').unsupported().pure().gap('most.basic', 'Most[{1, 2, 3}]', { expected: '{1, 2}' }).done(),
    feature('Take', 'list').unsupported().pure().gap('take.2', 'Take[{1, 2, 3, 4}, 2]', { expected: '{1, 2}' }).done(),
    feature('Drop', 'list').unsupported().pure().gap('drop.2', 'Drop[{1, 2, 3, 4}, 2]', { expected: '{3, 4}' }).done(),
    feature('Reverse', 'list').unsupported().pure().gap('reverse.3', 'Reverse[{1, 2, 3}]', { expected: '{3, 2, 1}' }).done(),
    feature('Sort', 'list').unsupported().pure().gap('sort.3', 'Sort[{3, 1, 2}]', { expected: '{1, 2, 3}' }).done(),
    feature('MemberQ', 'list').unsupported().pure().gap('memberq.2', 'MemberQ[{1, 2, 3}, 2]', { expected: 'True' }).done(),
    feature('IdentityMatrix', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('idmat.2', 'IdentityMatrix[2]', { expected: '{{1, 0}, {0, 1}}' })
        .done(),
    feature('Dimensions', 'linear_algebra').unsupported().pure().gap('dims.2x2', 'Dimensions[{{1, 2}, {3, 4}}]', { expected: '{2, 2}' }).done(),
    feature('Binomial', 'number_theory').unsupported().pure().gap('binomial.52', 'Binomial[5, 2]', { expected: '10' }).done(),
    feature('LCM', 'number_theory').unsupported().pure().gap('lcm.46', 'LCM[4, 6]', { expected: '12' }).done(),
    feature('Min', 'arithmetic').unsupported().pure().gap('min.3', 'Min[3, 1, 2]', { expected: '1' }).done(),
    feature('Sign', 'arithmetic').unsupported().pure().gap('sign.neg', 'Sign[-3]', { expected: '-1' }).done(),
    feature('Round', 'arithmetic').unsupported().pure().gap('round.2_5', 'Round[2.5]', { expected: '2' }).done(),
    feature('Ceiling', 'arithmetic').unsupported().pure().gap('ceiling.2_1', 'Ceiling[2.1]', { expected: '3' }).done(),
    feature('Re', 'complex').unsupported().pure().gap('re.i', 'Re[I]', { expected: '0' }).done(),
    feature('Im', 'complex').unsupported().pure().gap('im.i', 'Im[I]', { expected: '1' }).done(),
    feature('Conjugate', 'complex').unsupported().pure().gap('conj.i', 'Conjugate[I]', { expected: '-I' }).done(),
    feature('SameQ', 'comparison')
        .partial('SILENT WRONG: === lowers to Equal for symbols (x===x → Equal[x,x]); SameQ[1,1] unevaluated; numeric 1===1 → 1')
        .pure()
        .eval('sameq.num_infix', '1 === 1', 'True')
        .gap('sameq.head', 'SameQ[1, 1]', { expected: 'True' })
        .gap('sameq.sym_infix', 'x === x', { expected: 'True', notes: 'currently Equal[x, x]' })
        .done(),
    feature('Clear', 'session')
        .unsupported('SILENT WRONG: Clear[x] returns x')
        .stateful()
        .gap('clear.strip', 'Clear[x]', { expected: 'Null', notes: 'currently returns x' })
        .done(),
    feature('Timing', 'meta')
        .unsupported('SILENT WRONG: Timing[1+1] → Timing[2] (arg evaluated, no timing pair)')
        .effectful()
        .gap('timing.plus', 'Timing[1 + 1]', { expected: '{0., 2}' })
        .done(),
    feature('ToString', 'string')
        .unsupported('args evaluated first: ToString[1+1] → ToString[2]')
        .pure()
        .gap('tostring.plus', 'ToString[1 + 1]', { expected: '"2"' })
        .done(),
    feature('ToExpression', 'string').unsupported().pure().gap('toexpression.plus', 'ToExpression["1+1"]', { expected: '2' }).done(),
    feature('LaplaceTransform', 'calculus')
        .unsupported('nested/garbage ROCUnknown re-wrapping')
        .pure()
        .gap('laplace.exp', 'LaplaceTransform[Exp[-a*t], t, s]', { expected: '1/(a + s)' })
        .done(),
    feature('FourierTransform', 'calculus')
        .unsupported('SILENT WRONG: Exp[-x^2] becomes Exp[x^2] in residual form')
        .pure()
        .gap('fourier.gauss', 'FourierTransform[Exp[-x^2], x, k]', { expected: 'Sqrt[Pi]*Exp[-k^2/4]' })
        .done(),
    feature('Eliminate', 'solve')
        .planned()
        .pure()
        .gap('eliminate.basic', 'Eliminate[{x + y == 1, x - y == 0}, y]', { expected: 'x == 1/2' })
        .done(),
    feature('FindInstance', 'solve')
        .planned()
        .pure()
        .gap('findinstance.circle', 'FindInstance[x^2 + y^2 == 1, {x, y}]', { expected: '{{x -> 1, y -> 0}}' })
        .done(),
    feature('ParametricPlot', 'plot')
        .unsupported()
        .effectful()
        .gap('parametric.circle', 'ParametricPlot[{Cos[t], Sin[t]}, {t, 0, 2}]', { expected: '<svg' })
        .done(),
    feature('ContourPlot', 'plot')
        .unsupported()
        .effectful()
        .gap('contour.circle', 'ContourPlot[x^2 + y^2, {x, -1, 1}, {y, -1, 1}]', { expected: '<svg' })
        .done(),
    feature('JuxtapositionTimes', 'parse')
        .unsupported('implicit Times often becomes arg-splitting in D/Integrate/Collect (SILENT WRONG)')
        .pure()
        .gap('juxt.d', 'D[x y, x]', { expected: 'y', notes: 'currently D[x, y, x]' })
        .gap('juxt.collect', 'Collect[x^2 + 2*x*y + y^2, x]', { expected: 'x^2 + 2*x*y + y^2' })
        .done(),
    feature('Dt', 'calculus').unsupported().pure().gap('dt.x2', 'Dt[x^2]', { expected: '2*x*Dt[x]' }).done(),
    feature('Derivative', 'calculus').unsupported().pure().gap('derivative.sin', 'Derivative[1][Sin][x]', { expected: 'Cos[x]' }).done(),
    feature('Evaluate', 'hold')
        .unsupported('Hold args already evaluated: Evaluate[Hold[1+1]] → Evaluate[Hold[2]]')
        .pure()
        .gap('evaluate.hold', 'Evaluate[Hold[1 + 1]]', { expected: '2' })
        .done(),
    feature('ReleaseHold', 'hold')
        .unsupported('Hold already forced: ReleaseHold[Hold[1+1]] → ReleaseHold[Hold[2]]')
        .pure()
        .gap('releasehold.plus', 'ReleaseHold[Hold[1 + 1]]', { expected: '2' })
        .done(),
    feature('Unevaluated', 'hold')
        .unsupported('SILENT WRONG: Unevaluated[1+1] → Unevaluated[2]')
        .pure()
        .gap('unevaluated.plus', 'Unevaluated[1 + 1]', { expected: 'Unevaluated[1 + 1]' })
        .done(),
    feature('Catch', 'control').unsupported().pure().gap('catch.throw', 'Catch[Throw[2]]', { expected: '2' }).done(),
    feature('Quiet', 'meta').unsupported().pure().gap('quiet.div0', 'Quiet[1/0]', { expected: 'ComplexInfinity' }).done(),
    feature('Select', 'list')
        .unsupported('SILENT WRONG: Select[{1,2,3,4},EvenQ] → EvenQ')
        .pure()
        .gap('select.evenq', 'Select[{1, 2, 3, 4}, EvenQ]', { expected: '{2, 4}', notes: 'currently returns EvenQ' })
        .done(),
    feature('Cases', 'list')
        .unsupported('SILENT WRONG: Cases[{1,2,3},_Integer] → Integer (Blank stripped)')
        .pure()
        .gap('cases.integer', 'Cases[{1, 2, 3}, _Integer]', { expected: '{1, 2, 3}', notes: 'currently returns Integer' })
        .done(),
    feature('Count', 'list').unsupported().pure().gap('count.1', 'Count[{1, 1, 2}, 1]', { expected: '2' }).done(),
    feature('Partition', 'list').unsupported().pure().gap('partition.2', 'Partition[{1, 2, 3, 4}, 2]', { expected: '{{1, 2}, {3, 4}}' }).done(),
    feature('Union', 'list').unsupported().pure().gap('union.basic', 'Union[{1, 2}, {2, 3}]', { expected: '{1, 2, 3}' }).done(),
    feature('Intersection', 'list').unsupported().pure().gap('intersection.basic', 'Intersection[{1, 2}, {2, 3}]', { expected: '{2}' }).done(),
    feature('FreeQ', 'list').unsupported().pure().gap('freeq.3', 'FreeQ[{1, 2}, 3]', { expected: 'True' }).done(),
    feature('Position', 'list').unsupported().pure().gap('position.1', 'Position[{1, 2, 1}, 1]', { expected: '{{1}, {3}}' }).done(),
    feature('Extract', 'list').unsupported().pure().gap('extract.2', 'Extract[{1, 2, 3}, 2]', { expected: '2' }).done(),
    feature('PadLeft', 'list').unsupported().pure().gap('padleft.4', 'PadLeft[{1, 2}, 4]', { expected: '{0, 0, 1, 2}' }).done(),
    feature('Riffle', 'list').unsupported().pure().gap('riffle.ab', 'Riffle[{1, 2}, {a, b}]', { expected: '{1, a, 2, b}' }).done(),
    feature('Accumulate', 'list').unsupported().pure().gap('accumulate.3', 'Accumulate[{1, 2, 3}]', { expected: '{1, 3, 6}' }).done(),
    feature('Differences', 'list').unsupported().pure().gap('differences.3', 'Differences[{1, 4, 9}]', { expected: '{3, 5}' }).done(),
    feature('Thread', 'function').unsupported().pure().gap('thread.rule', 'Thread[{a, b} -> {1, 2}]', { expected: '{a -> 1, b -> 2}' }).done(),
    feature('Through', 'function')
        .unsupported('SILENT WRONG parse reshape: Through[{Sin,Cos}[0]] → Through[{Sin, Cos}, 0]')
        .pure()
        .gap('through.sincos', 'Through[{Sin, Cos}[0]]', { expected: '{0, 1}' })
        .done(),
    feature('StringTake', 'string').unsupported().pure().gap('stringtake.2', 'StringTake["abcd", 2]', { expected: '"ab"' }).done(),
    feature('StringReverse', 'string').unsupported().pure().gap('stringreverse.ab', 'StringReverse["ab"]', { expected: '"ba"' }).done(),
    feature('MatrixRank', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('matrixrank.rank1', 'MatrixRank[{{1, 2}, {2, 4}}]', { expected: '1' })
        .done(),
    feature('Eigenvalues', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('eigenvalues.diag', 'Eigenvalues[{{1, 0}, {0, 2}}]', { expected: '{2, 1}' })
        .done(),
    feature('DiagonalMatrix', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('diagmat.12', 'DiagonalMatrix[{1, 2}]', { expected: '{{1, 0}, {0, 2}}' })
        .done(),
    feature('Tr', 'linear_algebra').unsupported().pure().gap('tr.2x2', 'Tr[{{1, 2}, {3, 4}}]', { expected: '5' }).done(),
    feature('Norm', 'linear_algebra').unsupported().pure().gap('norm.34', 'Norm[{3, 4}]', { expected: '5' }).done(),
    feature('Cancel', 'algebra')
        .unsupported('SILENT WRONG rewrite of (x^2-1)/(x-1) into Cancel[-((-1+x)^-1)+…]')
        .pure()
        .gap('cancel.x2m1', 'Cancel[(x^2 - 1)/(x - 1)]', { expected: '1 + x' })
        .done(),
    feature('Variables', 'algebra').unsupported().pure().gap('variables.xyz', 'Variables[x*y + z]', { expected: '{x, y, z}' }).done(),
    feature('Numerator', 'algebra').unsupported().pure().gap('numerator.half', 'Numerator[1/2]', { expected: '1' }).done(),
    feature('Denominator', 'algebra').unsupported().pure().gap('denominator.34', 'Denominator[3/4]', { expected: '4' }).done(),
    feature('Together', 'algebra').unsupported().pure().gap('together.xy', 'Together[1/x + 1/y]', { expected: '(x + y)/(x*y)' }).done(),
    feature('Apart', 'algebra').unsupported().pure().gap('apart.partial', 'Apart[1/(x*(x + 1))]', { expected: '1/x - 1/(1 + x)' }).done(),
    feature('Coefficient', 'algebra').unsupported().pure().gap('coefficient.x', 'Coefficient[x^2 + 3*x, x]', { expected: '3' }).done(),
    feature('Exponent', 'algebra').unsupported().pure().gap('exponent.x3', 'Exponent[x^3 + x, x]', { expected: '3' }).done(),
    feature('PolynomialGCD', 'algebra')
        .unsupported()
        .pure()
        .gap('polygcd.basic', 'PolynomialGCD[x^2 - 1, x - 1]', { expected: '-1 + x' })
        .done(),
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
    feature('Gamma', 'special').unsupported().pure().gap('gamma.5', 'Gamma[5]', { expected: '24' }).done(),
    feature('Zeta', 'special').unsupported().pure().gap('zeta.2', 'Zeta[2]', { expected: 'Pi^2/6' }).done(),
    feature('Erf', 'special').unsupported().pure().gap('erf.0', 'Erf[0]', { expected: '0' }).done(),
    feature('ArcSin', 'elementary').unsupported().pure().gap('arcsin.0', 'ArcSin[0]', { expected: '0' }).done(),
    feature('Sinh', 'elementary').unsupported().pure().gap('sinh.0', 'Sinh[0]', { expected: '0' }).done(),
    feature('Cosh', 'elementary').unsupported().pure().gap('cosh.0', 'Cosh[0]', { expected: '1' }).done(),
    feature('LogE', 'elementary')
        .unsupported('Log[E] remains unevaluated (canonical Log[E]→1 missing)')
        .pure()
        .gap('log.e', 'Log[E]', { expected: '1' })
        .done(),
    feature('BitAnd', 'bitwise').unsupported().pure().gap('bitand.63', 'BitAnd[6, 3]', { expected: '2' }).done(),
    feature('IntegerDigits', 'number_theory')
        .unsupported()
        .pure()
        .gap('integerdigits.123', 'IntegerDigits[123]', { expected: '{1, 2, 3}' })
        .done(),
    feature('FromDigits', 'number_theory').unsupported().pure().gap('fromdigits.123', 'FromDigits[{1, 2, 3}]', { expected: '123' }).done(),
    feature('Boole', 'logic')
        .unsupported('SILENT WRONG: Boole[True] → Boole[] (True stripped); Boole[2>1] → Boole[1] (no 0/1 coerce)')
        .pure()
        .gap('boole.true', 'Boole[True]', { expected: '1', notes: 'currently Boole[]' })
        .gap('boole.pred', 'Boole[2 > 1]', { expected: '1', notes: 'currently Boole[1]' })
        .done(),
    feature('UnitStep', 'special')
        .unsupported('UnitStep[1] unevaluated; D[UnitStep[x],x] host stack-overflow crash — keep as gap only')
        .pure()
        .gap('unitstep.1', 'UnitStep[1]', { expected: '1' })
        .gap('unitstep.deriv_crash', 'D[UnitStep[x], x]', {
            expected: 'DiracDelta[x]',
            notes: 'host crash (stack overflow) observed — do not promote to eval',
        })
        .done(),
    feature('HeavisideTheta', 'special').unsupported().pure().gap('heaviside.1', 'HeavisideTheta[1]', { expected: '1' }).done(),
    feature('Fourier', 'calculus').unsupported().pure().gap('fourier.vec', 'Fourier[{1, 2, 3, 4}]', { expected: '...' }).done(),
    feature('InverseFourier', 'calculus')
        .unsupported()
        .pure()
        .gap('inversefourier.impulse', 'InverseFourier[{1, 0, 0, 0}]', { expected: '...' })
        .done(),
    feature('Fit', 'fit').unsupported().pure().gap('fit.linear', 'Fit[{1, 2}, {1, x}, x]', { expected: '...' }).done(),
    feature('FindFit', 'fit').unsupported().pure().gap('findfit.ax', 'FindFit[{1, 4}, {a*x}, {a}, x]', { expected: '{a -> 2.}' }).done(),
    feature('PossibleZeroQ', 'predicates').unsupported().pure().gap('possiblezeroq.0', 'PossibleZeroQ[0]', { expected: 'True' }).done(),
    feature('NumericQ', 'predicates').unsupported().pure().gap('numericq.1', 'NumericQ[1]', { expected: 'True' }).done(),
    feature('IntegerQ', 'predicates').unsupported().pure().gap('integerq.1', 'IntegerQ[1]', { expected: 'True' }).done(),
    feature('UnsameQ', 'comparison').unsupported().pure().gap('unsameq.12', 'UnsameQ[1, 2]', { expected: 'True' }).done(),
    feature('Do', 'control')
        .unsupported('SILENT WRONG: Do[1,{3}] → {3}; Do[i,{i,3}] → {i, 3} (body stripped like Table)')
        .stateful()
        .gap('do.strip', 'Do[1, {3}]', { expected: 'Null', notes: 'currently returns {3}' })
        .done(),
    feature('While', 'control')
        .unsupported('SILENT WRONG: While[False,1] → 1 (body runs / False atom broken)')
        .stateful()
        .gap('while.false', 'While[False, 1]', { expected: 'Null', notes: 'currently returns 1' })
        .done(),
    feature('For', 'control')
        .unsupported('oak error node on For[i=1,i<3,i++,i]')
        .stateful()
        .gap('for.basic', 'For[i = 1, i < 3, i++, i]', { expected: 'Null' })
        .done(),
    feature('MatchQ', 'pattern')
        .unsupported('SILENT WRONG: Blank stripped — MatchQ[1,_Integer]→MatchQ[1,Integer]; MatchQ[f[1],f[_]]→MatchQ[f[1],f[]]')
        .pure()
        .gap('matchq.integer', 'MatchQ[1, _Integer]', { expected: 'True', notes: 'currently MatchQ[1, Integer]' })
        .done(),
    feature('Condition', 'pattern')
        .unsupported('Blank stripped: Condition[x_,x>0] → Condition[x, Greater[x, 0]]')
        .pure()
        .gap('condition.pattern', 'MatchQ[2, x_ /; x > 0]', { expected: 'True' })
        .done(),
    feature('Association', 'association')
        .unsupported('Association[…] unevaluated; <|…|> oak error node')
        .pure()
        .gap('assoc.head', 'Association[a -> 1, b -> 2]', { expected: '<|a -> 1, b -> 2|>' })
        .gap('assoc.literal', '<|a -> 1|>', { expected: '<|a -> 1|>', notes: 'oak error node' })
        .done(),
    feature('Residue', 'calculus')
        .partial(
            'some simple poles OK (1/z, Exp[z]/z); Residue[1/(z-1),{z,1}] → 0 SILENT WRONG (expect 1); partial fractions often unevaluated',
        )
        .pure()
        .eval('residue.1_z', 'Residue[1/z, {z, 0}]', '1')
        .eval('residue.exp_z', 'Residue[Exp[z]/z, {z, 0}]', '1')
        .gap('residue.shift', 'Residue[1/(z - 1), {z, 1}]', { expected: '1', notes: 'currently returns 0' })
        .done(),
    feature('InputForm', 'forms')
        .unsupported('SILENT WRONG eval-before-wrap: InputForm[1+1]→InputForm[2]; Hold still forced inside')
        .pure()
        .gap('inputform.plus', 'InputForm[1 + 1]', { expected: 'InputForm[1 + 1]' })
        .done(),
    feature('FullForm', 'forms').unsupported().pure().gap('fullform.plus', 'FullForm[1 + x]', { expected: 'Plus[1, x]' }).done(),
    feature('TeXForm', 'forms').unsupported().pure().gap('texform.x2', 'TeXForm[x^2]', { expected: 'x^2' }).done(),
    feature('Trace', 'meta')
        .unsupported('SILENT WRONG: Trace[1+1] → Trace[2]')
        .pure()
        .gap('trace.plus', 'Trace[1 + 1]', { expected: '{{1+1,2}}' })
        .done(),
    feature('Assert', 'meta')
        .unsupported('SILENT WRONG: Assert[True] → Assert[] (True stripped)')
        .pure()
        .gap('assert.true', 'Assert[True]', { expected: 'Null', notes: 'currently Assert[]' })
        .done(),
    feature('MessageName', 'meta')
        .unsupported('SILENT WRONG: Message[f::x] → Message[f, x] (:: MessageName broken)')
        .pure()
        .gap('message.colon', 'Message[f::x]', { expected: 'Null', notes: 'currently Message[f, x]' })
        .done(),
    feature('Information', 'meta')
        .unsupported('SILENT WRONG: ??Plus / ?Plus → Plus')
        .pure()
        .gap('info.qq', '??Plus', { expected: '...', notes: 'currently returns Plus' })
        .done(),
    feature('Total', 'list').unsupported().pure().gap('total.3', 'Total[{1, 2, 3}]', { expected: '6' }).done(),
    feature('Mean', 'stats').unsupported().pure().gap('mean.3', 'Mean[{1, 2, 3}]', { expected: '2' }).done(),
    feature('Append', 'list').unsupported().pure().gap('append.3', 'Append[{1, 2}, 3]', { expected: '{1, 2, 3}' }).done(),
    feature('Prepend', 'list').unsupported().pure().gap('prepend.1', 'Prepend[{2, 3}, 1]', { expected: '{1, 2, 3}' }).done(),
    feature('DeleteDuplicates', 'list').unsupported().pure().gap('deletedup.112', 'DeleteDuplicates[{1, 1, 2}]', { expected: '{1, 2}' }).done(),
    feature('Cross', 'linear_algebra').unsupported().pure().gap('cross.ijk', 'Cross[{1, 0, 0}, {0, 1, 0}]', { expected: '{0, 0, 1}' }).done(),
    feature('Array', 'list').unsupported().pure().gap('array.f3', 'Array[f, 3]', { expected: '{f[1], f[2], f[3]}' }).done(),
    feature('ConstantArray', 'list').unsupported().pure().gap('constarray.0', 'ConstantArray[0, 3]', { expected: '{0, 0, 0}' }).done(),
    feature('StringJoin', 'string').unsupported().pure().gap('stringjoin.ab', 'StringJoin["a", "b"]', { expected: '"ab"' }).done(),
    feature('Characters', 'string').unsupported().pure().gap('characters.ab', 'Characters["ab"]', { expected: '{"a", "b"}' }).done(),
    feature('ArcTan', 'elementary').unsupported().pure().gap('arctan.1', 'ArcTan[1]', { expected: 'Pi/4' }).done(),
    feature('BitOr', 'bitwise').unsupported().pure().gap('bitor.12', 'BitOr[1, 2]', { expected: '3' }).done(),
    feature('Prime', 'number_theory').unsupported().pure().gap('prime.10', 'Prime[10]', { expected: '29' }).done(),
    feature('FactorInteger', 'number_theory')
        .unsupported()
        .pure()
        .gap('factorinteger.12', 'FactorInteger[12]', { expected: '{{2, 2}, {3, 1}}' })
        .done(),
    feature('Infinity', 'constant')
        .partial('symbol retained; arithmetic with Infinity incomplete')
        .pure()
        .eval('inf.symbol', 'Infinity', 'Infinity')
        .done(),
    feature('NDSolve', 'solve')
        .planned("oak error on y'[x] sugar")
        .pure()
        .gap('ndsolve.exp', "NDSolve[{y'[x] == y[x], y[0] == 1}, y, {x, 0, 1}]", { expected: '...' })
        .done(),
    feature('FindMinimum', 'solve').planned().pure().gap('findminimum.x2', 'FindMinimum[x^2, {x, 1}]', { expected: '{0., {x -> 0.}}' }).done(),
    feature('PolarPlot', 'plot').unsupported().effectful().gap('polarplot.1', 'PolarPlot[1, {t, 0, 2}]', { expected: '<svg' }).done(),
    feature('RegionPlot', 'plot')
        .unsupported()
        .effectful()
        .gap('regionplot.disk', 'RegionPlot[x^2 + y^2 < 1, {x, -1, 1}, {y, -1, 1}]', { expected: '<svg' })
        .done(),
    feature('BarChart', 'plot').unsupported().effectful().gap('barchart.3', 'BarChart[{1, 2, 3}]', { expected: '<svg' }).done(),
    feature('Composition', 'function')
        .unsupported()
        .pure()
        .gap('composition.sincos', 'Composition[Sin, Cos][0]', { expected: 'Sin[1]' })
        .done(),
    feature('AttributesPlus', 'attributes')
        .planned('Attributes[Plus] unevaluated (Orderless/Flat/Listable contract)')
        .pure()
        .gap('attr.plus', 'Attributes[Plus]', { expected: '{Flat, Listable, NumericFunction, OneIdentity, Orderless, Protected}' })
        .done(),
    feature('Prefix', 'parse').supported().pure().notes('f@x → f[x]').eval('prefix.fx', 'f@x', 'f[x]').done(),
    feature('Postfix', 'parse').supported().pure().notes('x//f → f[x]').eval('postfix.xf', 'x//f', 'f[x]').done(),
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
    feature('HoldComplete', 'hold')
        .unsupported('SILENT WRONG: HoldComplete[1+1] → HoldComplete[2]')
        .unevaluated()
        .gap('holdcomplete.plus', 'HoldComplete[1 + 1]', { expected: 'HoldComplete[1 + 1]', notes: 'currently HoldComplete[2]' })
        .done(),
    feature('Inactive', 'hold')
        .partial('Inactive[Plus][1,2] retained; Inactivate[1+2] forces arg first → Inactivate[3]')
        .unevaluated()
        .eval('inactive.plus', 'Inactive[Plus][1, 2]', 'Inactive[Plus][1, 2]')
        .gap('inactivate.plus', 'Inactivate[1 + 2]', { expected: 'Inactive[Plus][1, 2]', notes: 'currently Inactivate[3]' })
        .done(),
    feature('Activate', 'hold').unsupported().pure().gap('activate.plus', 'Activate[Inactive[Plus][1, 2]]', { expected: '3' }).done(),
    feature('PatternTest', 'pattern')
        .unsupported('SILENT WRONG: Cases[{1,a,2},_?NumberQ] → NumberQ (same Blank/? strip as MatchQ)')
        .pure()
        .gap('patterntest.numberq', 'Cases[{1, a, 2}, _?NumberQ]', { expected: '{1, 2}', notes: 'currently returns NumberQ' })
        .done(),
    feature('DSolveValue', 'solve')
        .planned("SILENT WRONG with y' sugar: DSolveValue[y'[x]==y[x],y[x],x] → x; plain form unevaluated")
        .pure()
        .gap('dsolvevalue.strip', "DSolveValue[y'[x] == y[x], y[x], x]", { expected: 'C[1]*Exp[x]', notes: 'currently returns x' })
        .done(),
    feature('AtomQ', 'predicates').unsupported().pure().gap('atomq.1', 'AtomQ[1]', { expected: 'True' }).done(),
    feature('NumberQ', 'predicates').unsupported().pure().gap('numberq.12', 'NumberQ[1.2]', { expected: 'True' }).done(),
    feature('EvenQ', 'predicates').unsupported().pure().gap('evenq.2', 'EvenQ[2]', { expected: 'True' }).done(),
    feature('Positive', 'predicates').unsupported().pure().gap('positive.3', 'Positive[3]', { expected: 'True' }).done(),
    feature('VectorQ', 'predicates').unsupported().pure().gap('vectorq.12', 'VectorQ[{1, 2}]', { expected: 'True' }).done(),
    feature('MatrixQ', 'predicates').unsupported().pure().gap('matrixq.row', 'MatrixQ[{{1, 2}}]', { expected: 'True' }).done(),
    feature('ListQ', 'predicates').unsupported().pure().gap('listq.1', 'ListQ[{1}]', { expected: 'True' }).done(),
    feature('StringQ', 'predicates').unsupported().pure().gap('stringq.a', 'StringQ["a"]', { expected: 'True' }).done(),
    feature('Arg', 'complex').unsupported().pure().gap('arg.i', 'Arg[I]', { expected: 'Pi/2' }).done(),
    feature('TrigReduce', 'simplify')
        .unsupported()
        .pure()
        .gap('trigreduce.sin2', 'TrigReduce[Sin[x]^2]', { expected: '(1 - Cos[2*x])/2' })
        .done(),
    feature('PowerExpand', 'simplify').unsupported().pure().gap('powerexpand.sqrt', 'PowerExpand[Sqrt[x^2]]', { expected: 'x' }).done(),
    feature('InverseLaplaceTransform', 'calculus')
        .unsupported()
        .pure()
        .gap('ilaplace.exp', 'InverseLaplaceTransform[1/(s + a), s, t]', { expected: 'Exp[-a*t]' })
        .done(),
    feature('Quantity', 'units').unsupported().pure().gap('quantity.m', 'Quantity[1, "Meters"]', { expected: 'Quantity[1, "Meters"]' }).done(),
    feature('Graph', 'graph').unsupported().pure().gap('graph.path', 'Graph[{1 -> 2, 2 -> 3}]', { expected: '...' }).done(),
    feature('UpSet', 'session').planned().stateful().gap('upset.basic', 'UpSet[f[x], 1]', { expected: '1' }).done(),
    feature('TagSet', 'session').planned('oak error on x/:f[x]=1').stateful().gap('tagset.basic', 'x /: f[x] = 1', { expected: '1' }).done(),
    feature('Unset', 'session').planned('oak error on a=.=').stateful().gap('unset.basic', 'a =.', { expected: 'Null' }).done(),
    feature('IndeterminateForms', 'arithmetic')
        .unsupported('SILENT WRONG: 0/0→0; Infinity-Infinity→0; 0^0→1 (MMA expects Indeterminate)')
        .pure()
        .gap('indet.0over0', '0/0', { expected: 'Indeterminate', notes: 'currently 0' })
        .gap('indet.inf_minus_inf', 'Infinity - Infinity', { expected: 'Indeterminate', notes: 'currently 0' })
        .gap('indet.0pow0', '0^0', { expected: 'Indeterminate', notes: 'currently 1' })
        .done(),
    feature('InequalityChain', 'comparison')
        .supported()
        .pure()
        .notes('nested relational ops evaluate via Athena compare-chain flattening')
        .eval('ineq.lt_chain', '1 < 2 < 3', 'True')
        .eval('ineq.mixed', '1 < 3 > 2', 'True')
        .done(),
    feature('UnsameQInfix', 'comparison')
        .partial('numeric 1=!=2 → 1; symbols lower to Unequal not UnsameQ')
        .pure()
        .eval('unsameq.num_infix', '1 =!= 2', 'True')
        .gap('unsameq.sym_infix', 'x =!= x', { expected: 'False', notes: 'currently Unequal[x, x]' })
        .done(),
    feature('DeleteCases', 'list')
        .unsupported('Blank stripped: DeleteCases[{1,a,2},_Integer] → DeleteCases[..., Integer]')
        .pure()
        .gap('deletecases.int', 'DeleteCases[{1, a, 2}, _Integer]', { expected: '{a}' })
        .done(),
    feature('MapIndexed', 'list')
        .unsupported('SILENT WRONG parse of #2& → MapIndexed[#, 2 &, …]')
        .pure()
        .gap('mapindexed.slot2', 'MapIndexed[#2 &, {a, b}]', { expected: '{{1}, {2}}' })
        .done(),
    feature('MapThread', 'list')
        .unsupported()
        .pure()
        .gap('mapthread.f', 'MapThread[f, {{1, 2}, {3, 4}}]', { expected: '{f[1, 3], f[2, 4]}' })
        .done(),
    feature('Assuming', 'simplify')
        .unsupported('Assuming retained but body not simplified under assumption')
        .pure()
        .gap('assuming.sqrt', 'Assuming[x > 0, Simplify[Sqrt[x^2]]]', { expected: 'x' })
        .done(),
    feature('Refine', 'simplify').unsupported().pure().gap('refine.sqrt', 'Refine[Sqrt[x^2], x > 0]', { expected: 'x' }).done(),
    feature('Eigenvectors', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('eigenvectors.diag', 'Eigenvectors[{{1, 0}, {0, 2}}]', { expected: '{{0, 1}, {1, 0}}' })
        .done(),
    feature('NullSpace', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('nullspace.rank1', 'NullSpace[{{1, 2}, {2, 4}}]', { expected: '{{-2}, {1}}' })
        .done(),
    feature('MatrixExp', 'linear_algebra')
        .unsupported()
        .pure()
        .gap('matrixexp.rot', 'MatrixExp[{{0, 1}, {-1, 0}}]', { expected: '...' })
        .done(),
    feature('PowerMod', 'number_theory').unsupported().pure().gap('powermod.basic', 'PowerMod[2, 10, 7]', { expected: '2' }).done(),
    feature('ChineseRemainder', 'number_theory')
        .unsupported()
        .pure()
        .gap('crt.basic', 'ChineseRemainder[{1, 2}, {3, 5}]', { expected: '7' })
        .done(),
    feature('StringTrim', 'string').unsupported().pure().gap('stringtrim.a', 'StringTrim[" a "]', { expected: '"a"' }).done(),
    feature('RGBColor', 'graphics').unsupported().pure().gap('rgb.red', 'RGBColor[1, 0, 0]', { expected: 'RGBColor[1, 0, 0]' }).done(),
    feature('Style', 'graphics').unsupported().pure().gap('style.bold', 'Style[x, Bold]', { expected: 'Style[x, Bold]' }).done(),
    feature('CubeRootPow', 'arithmetic')
        .unsupported('SILENT WRONG parse/precedence: (-8)^(1/3) → -8^1/3')
        .pure()
        .gap('cuberoot.neg8', '(-8)^(1/3)', { expected: '-2', notes: 'currently -8^1/3' })
        .done(),
    feature('Maximize', 'solve')
        .planned('SILENT WRONG: Maximize[-x^2,x] → Maximize[x^2, x] (unary minus stripped)')
        .pure()
        .gap('maximize.neg_quad', 'Maximize[-x^2, x]', { expected: '{0, {x -> 0}}', notes: 'currently Maximize[x^2, x]' })
        .done(),
    feature('Minimize', 'solve').planned().pure().gap('minimize.quad', 'Minimize[x^2 + 1, x]', { expected: '{1, {x -> 0}}' }).done(),
    feature('Xor', 'logic')
        .unsupported('SILENT WRONG: Xor[True,False] → Xor[] (True/False stripped)')
        .pure()
        .gap('xor.tf', 'Xor[True, False]', { expected: 'True', notes: 'currently Xor[]' })
        .done(),
    feature('Implies', 'logic')
        .unsupported('SILENT WRONG: Implies[True,False] → Implies[]')
        .pure()
        .gap('implies.tf', 'Implies[True, False]', { expected: 'False', notes: 'currently Implies[]' })
        .done(),
    feature('TrueQ', 'predicates')
        .unsupported('SILENT WRONG: TrueQ[True]→TrueQ[]; TrueQ[1==1]→TrueQ[1]')
        .pure()
        .gap('trueq.equal', 'TrueQ[1 == 1]', { expected: 'True', notes: 'currently TrueQ[1]' })
        .done(),
    feature('BooleanQ', 'predicates')
        .unsupported('SILENT WRONG: BooleanQ[True] → BooleanQ[]')
        .pure()
        .gap('booleanq.true', 'BooleanQ[True]', { expected: 'True', notes: 'currently BooleanQ[]' })
        .done(),
    feature('DAbs', 'calculus')
        .partial('D[Abs[x],x] → Abs[x]/x form (x^-1*Abs[x]); acceptable rewrite, not Sign[x]')
        .pure()
        .eval('dabs.x', 'D[Abs[x], x]', 'x^-1*Abs[x]')
        .done(),
    feature('ComplexMul', 'complex')
        .unsupported('(1+I)*(1-I) → 1+-(I^2) not folded to 2')
        .pure()
        .gap('complexmul.conj', '(1 + I)*(1 - I)', { expected: '2', notes: 'currently 1 + -(I^2)' })
        .done(),
    feature('SqrtRational', 'arithmetic').supported().pure().eval('sqrt.9_4', 'Sqrt[9/4]', '3/2').done(),
    feature('RationalAdd', 'arithmetic').supported().pure().eval('rational.add', '1/2 + 1/3', '5/6').done(),
    feature('Fibonacci', 'number_theory').unsupported().pure().gap('fibonacci.10', 'Fibonacci[10]', { expected: '55' }).done(),
    feature('CubeRoot', 'arithmetic').unsupported().pure().gap('cuberoot.m8', 'CubeRoot[-8]', { expected: '-2' }).done(),
    feature('Surd', 'arithmetic').unsupported().pure().gap('surd.m8_3', 'Surd[-8, 3]', { expected: '-2' }).done(),
    feature('BesselJ', 'special').unsupported().pure().gap('besselj.10', 'BesselJ[1, 0]', { expected: '0' }).done(),
    feature('LegendreP', 'special').unsupported().pure().gap('legendrep.2', 'LegendreP[2, x]', { expected: '(-1 + 3*x^2)/2' }).done(),
    feature('GammaHalf', 'special').unsupported().pure().gap('gamma.half', 'Gamma[1/2]', { expected: 'Sqrt[Pi]' }).done(),
    feature('ZetaZero', 'special').unsupported().pure().gap('zeta.0', 'Zeta[0]', { expected: '-1/2' }).done(),
    feature('DynamicModule', 'session')
        .unsupported('SILENT WRONG: DynamicModule[{x=1},x] → DynamicModule[{1}, x]')
        .stateful()
        .gap('dynamicmodule.bind', 'DynamicModule[{x = 1}, x]', { expected: '1', notes: 'currently DynamicModule[{1}, x]' })
        .done(),
    feature('FileNameJoin', 'io').unsupported().pure().gap('filenamejoin.ab', 'FileNameJoin[{"a", "b"}]', { expected: '...' }).done(),
    feature('Normalize', 'linear_algebra').unsupported().pure().gap('normalize.34', 'Normalize[{3, 4}]', { expected: '{3/5, 4/5}' }).done(),
    feature('Curl', 'calculus')
        .unsupported('SILENT WRONG: Curl[{-y,x},{x,y}] → {} (expect 2)')
        .pure()
        .gap('curl.2d', 'Curl[{-y, x}, {x, y}]', { expected: '2', notes: 'currently {}' })
        .done(),
    feature('Grad', 'calculus').unsupported().pure().gap('grad.xy', 'Grad[x*y, {x, y}]', { expected: '{y, x}' }).done(),
    feature('Div', 'calculus').unsupported().pure().gap('div.xy', 'Div[{x, y}, {x, y}]', { expected: '2' }).done(),
    feature('BlankSequence', 'pattern')
        .unsupported('SILENT WRONG: MatchQ[f[1,2],f[__]] → MatchQ[f[1,2],f[]] (__ stripped)')
        .pure()
        .gap('blankseq.match', 'MatchQ[f[1, 2], f[__]]', { expected: 'True', notes: 'currently MatchQ[f[1, 2], f[]]' })
        .done(),
    feature('BlankNullSequence', 'pattern')
        .unsupported('SILENT WRONG: ___ stripped to empty args')
        .pure()
        .gap('blanknullseq.match', 'MatchQ[f[], f[___]]', { expected: 'True', notes: 'currently MatchQ[f[], f[]]' })
        .done(),
    feature('ReplaceList', 'rule')
        .unsupported('Pattern stripped; may return RHS vars like {a,b,c} instead of matches')
        .pure()
        .gap('replacelist.pair', 'ReplaceList[{1, 2}, {a_, b_} :> a + b]', { expected: '{3}' })
        .done(),
    feature('ScientificLiteral', 'parse')
        .unsupported('oak error on 1*^3 and 2.5*^-2')
        .pure()
        .gap('sci.1e3', '1*^3', { expected: '1000' })
        .gap('sci.2p5em2', '2.5*^-2', { expected: '0.025' })
        .done(),
    feature('BaseLiteral', 'parse')
        .unsupported('oak error on 16^^FF and 2^^1010')
        .pure()
        .gap('base.hex', '16^^FF', { expected: '255' })
        .gap('base.bin', '2^^1010', { expected: '10' })
        .done(),
    feature('EuclideanDistance', 'geometry')
        .unsupported()
        .pure()
        .gap('euclid.345', 'EuclideanDistance[{0, 0}, {3, 4}]', { expected: '5' })
        .done(),
    feature('Area', 'geometry').unsupported().pure().gap('area.disk', 'Area[Disk[]]', { expected: 'Pi' }).done(),
    feature('Nearest', 'list').unsupported().pure().gap('nearest.3', 'Nearest[{1, 2, 4}, 3]', { expected: '{2, 4}' }).done(),
    feature('Counts', 'list').unsupported().pure().gap('counts.aab', 'Counts[{a, a, b}]', { expected: '<|a -> 2, b -> 1|>' }).done(),
    feature('ParallelEvaluate', 'parallel')
        .unsupported('SILENT WRONG eval-early: ParallelEvaluate[1+1] → ParallelEvaluate[2]')
        .effectful()
        .gap('paralleleval.plus', 'ParallelEvaluate[1 + 1]', { expected: '2', notes: 'currently ParallelEvaluate[2]' })
        .done(),
    feature('PrependTo', 'session')
        .unsupported('SILENT WRONG: PrependTo[x={1},0] → PrependTo[{1},0] (loses symbol)')
        .stateful()
        .gap('prependto.x', 'x = {1}; PrependTo[x, 0]; x', { expected: '{0, 1}' })
        .done(),
    feature('Construct', 'function').unsupported().pure().gap('construct.f12', 'Construct[f, 1, 2]', { expected: 'f[1, 2]' }).done(),
    feature('GraphDistance', 'graph')
        .unsupported()
        .pure()
        .gap('graphdistance.path', 'GraphDistance[Graph[{1 -> 2, 2 -> 3}], 1, 3]', { expected: '2' })
        .done(),
    feature('ZTransform', 'calculus')
        .unsupported('SILENT WRONG: nested ZTransform[…, ROCUnknown] re-wrapping (same family as LaplaceTransform)')
        .pure()
        .gap('ztransform.n', 'ZTransform[n, n, z]', {
            expected: 'z/(-1 + z)^2',
            notes: 'currently ZTransform[ZTransform[n, n, z], {n, z}, ROCUnknown]',
        })
        .done(),
    feature('InverseZTransform', 'calculus')
        .unsupported()
        .pure()
        .gap('iztrans.step', 'InverseZTransform[z/(z - 1), z, n]', { expected: '1' })
        .done(),
    feature('ApplySequence', 'function')
        .partial('Apply[Sequence,{1,2}] stays Sequence[1,2]; infix Sequence@@ may stay Apply')
        .pure()
        .gap('apply.sequence_head', 'Apply[Sequence, {1, 2}]', { expected: '{1, 2}', notes: 'currently Sequence[1, 2]' })
        .gap('apply.sequence_infix', 'Sequence @@ {1, 2}', { expected: '{1, 2}' })
        .done(),
    feature('Element', 'predicates').unsupported().pure().gap('element.int', 'Element[1, Integers]', { expected: 'True' }).done(),
    feature('SymmetricMatrixQ', 'predicates')
        .unsupported()
        .pure()
        .gap('symmatq.yes', 'SymmetricMatrixQ[{{1, 2}, {2, 1}}]', { expected: 'True' })
        .done(),
    feature('Discriminant', 'algebra').unsupported().pure().gap('discriminant.quad', 'Discriminant[x^2 + x + 1, x]', { expected: '-3' }).done(),
    feature('Resultant', 'algebra').unsupported().pure().gap('resultant.basic', 'Resultant[x^2 - 1, x - 1, x]', { expected: '0' }).done(),
    feature('PolynomialRemainder', 'algebra')
        .unsupported()
        .pure()
        .gap('polyrem.basic', 'PolynomialRemainder[x^3 + 1, x + 1, x]', { expected: '0' })
        .done(),
    feature('ListConvolve', 'list').unsupported().pure().gap('listconvolve.basic', 'ListConvolve[{1, 2}, {3, 4}]', { expected: '{11}' }).done(),
    feature('Hash', 'string').unsupported().pure().gap('hash.a', 'Hash["a"]', { expected: '...' }).done(),
    feature('ExportString', 'io')
        .unsupported()
        .pure()
        .gap('exportstring.csv', 'ExportString[{{1, 2}}, "CSV"]', { expected: '"1,2\\n"' })
        .done(),
    feature('ImportString', 'io').unsupported().pure().gap('importstring.csv', 'ImportString["1,2", "CSV"]', { expected: '{{1, 2}}' }).done(),
    feature('MemoryInUse', 'meta').unsupported().effectful().gap('memoryinuse.basic', 'MemoryInUse[]', { expected: '...' }).done(),
    feature('DollarVersion', 'meta').unsupported().pure().gap('version.atom', '$Version', { expected: '...' }).done(),
    feature('CompoundExpressionSet', 'session')
        .unsupported('SILENT WRONG: CompoundExpression[a=1,a] → a (no binding)')
        .stateful()
        .gap('compound.set', 'CompoundExpression[a = 1, a]', { expected: '1', notes: 'currently returns a' })
        .done(),
    feature('Interval', 'numeric')
        .unsupported()
        .pure()
        .gap('interval.basic', 'Interval[{1, 2}]', { expected: 'Interval[{1, 2}]', notes: 'echo only; no arithmetic' })
        .done(),
    feature('IntervalUnion', 'numeric')
        .unsupported()
        .pure()
        .gap('intervalunion.adj', 'IntervalUnion[Interval[{1, 2}], Interval[{2, 3}]]', { expected: 'Interval[{1, 3}]' })
        .done(),
    feature('IntervalIntersection', 'numeric')
        .unsupported()
        .pure()
        .gap('intervalintersection.overlap', 'IntervalIntersection[Interval[{1, 3}], Interval[{2, 4}]]', { expected: 'Interval[{2, 3}]' })
        .done(),
    feature('Around', 'numeric').unsupported().pure().gap('around.basic', 'Around[1.23, 0.01]', { expected: 'Around[1.23, 0.01]' }).done(),
    feature('Clip', 'arithmetic')
        .unsupported()
        .pure()
        .gap('clip.hi', 'Clip[5, {0, 1}]', { expected: '1' })
        .gap('clip.lo', 'Clip[-1, {0, 1}]', { expected: '0' })
        .done(),
    feature('Rescale', 'arithmetic').unsupported().pure().gap('rescale.mid', 'Rescale[0.5, {0, 1}, {-1, 1}]', { expected: '0' }).done(),
    feature('ArrayFlatten', 'list')
        .unsupported()
        .pure()
        .gap('arrayflatten.2x2', 'ArrayFlatten[{{{1, 2}}, {{3, 4}}}]', { expected: '{{1, 2}, {3, 4}}' })
        .done(),
    feature('TensorProduct', 'list')
        .unsupported()
        .pure()
        .gap('tensorproduct.vec', 'TensorProduct[{1, 2}, {3, 4}]', { expected: '{{3, 4}, {6, 8}}' })
        .done(),
    feature('HammingDistance', 'string')
        .unsupported()
        .pure()
        .gap('hamming.bits', 'HammingDistance[{1, 0, 1}, {1, 1, 0}]', { expected: '2' })
        .done(),
    feature('EditDistance', 'string')
        .unsupported()
        .pure()
        .gap('editdistance.kitten', 'EditDistance["kitten", "sitting"]', { expected: '3' })
        .done(),
    feature('Sinc', 'special')
        .unsupported()
        .pure()
        .gap('sinc.0', 'Sinc[0]', { expected: '1' })
        .gap('sinc.pi', 'Sinc[Pi]', { expected: '0' })
        .done(),
    feature('FresnelC', 'special').unsupported().pure().gap('fresnelc.inf', 'FresnelC[Infinity]', { expected: '1/2' }).done(),
    feature('InverseErf', 'special').unsupported().pure().gap('inverseerf.0', 'InverseErf[0]', { expected: '0' }).done(),
    feature('SquareWave', 'special')
        .unsupported()
        .pure()
        .gap('squarewave.sym', 'SquareWave[x]', { expected: 'SquareWave[x]', notes: 'echo; no numeric samples' })
        .done(),
    feature('FourierSequenceTransform', 'calculus')
        .unsupported()
        .pure()
        .gap('fst.a_n', 'FourierSequenceTransform[a^n, n, w]', { expected: '...' })
        .done(),
    feature('GeneratingFunction', 'calculus')
        .unsupported('Factorial head appears; no closed form')
        .pure()
        .gap('genfun.factorial', 'GeneratingFunction[n!, n, x]', {
            expected: '1/(1 - x)',
            notes: 'actually egf for n!; ogf different — record unevaluated',
        })
        .done(),
    feature('MapAll', 'list').unsupported().pure().gap('mapall.nest', 'MapAll[f, {1, {2}}]', { expected: 'f[{f[1], f[{f[2]}]}]' }).done(),
    feature('ComposeList', 'functional')
        .unsupported()
        .pure()
        .gap('composelist.fg', 'ComposeList[{f, g}, x]', { expected: '{x, f[x], g[f[x]]}' })
        .done(),
    feature('FilterRules', 'rule')
        .unsupported()
        .pure()
        .gap('filterrules.a', 'FilterRules[{a -> 1, b -> 2}, {a}]', { expected: '{a -> 1}' })
        .done(),
    feature('ReapSow', 'session')
        .unsupported('SILENT WRONG: Reap[Sow[1];Sow[2]] → Reap[Sow[2]] (CompoundExpression last-only + no Reap collect)')
        .stateful()
        .gap('reap.basic', 'Reap[Sow[1]]', { expected: '{1, {{1}}}' })
        .gap('reap.two', 'Reap[Sow[1]; Sow[2]]', { expected: '{2, {{1, 2}}}', notes: 'currently Reap[Sow[2]]' })
        .done(),
    feature('RandomInteger', 'random').unsupported().effectful().gap('randominteger.10', 'RandomInteger[10]', { expected: '...' }).done(),
    feature('SeedRandom', 'random')
        .unsupported('SeedRandom[1]; RandomInteger[5] → RandomInteger[5] (no seed / no draw)')
        .stateful()
        .gap('seedrandom.then', 'SeedRandom[1]; RandomInteger[5]', { expected: '...' })
        .done(),
    feature('PatternConditionDef', 'pattern')
        .unsupported('f[x_/;x>0]:=x oak error; f[x_?Positive]:=x; f[1] stays f[1]')
        .stateful()
        .gap('pattern.condition_def', 'f[x_ /; x > 0] := x; f[1]', { expected: '1', notes: 'oak error node' })
        .gap('pattern.patternTest_def', 'f[x_?Positive] := x; f[1]', { expected: '1', notes: 'currently f[1]' })
        .done(),
    feature('DateObject', 'datetime').unsupported().pure().gap('dateobject.ymd', 'DateObject[{2020, 1, 1}]', { expected: '...' }).done(),
    feature('DatePlus', 'datetime').unsupported().pure().gap('dateplus.today', 'DatePlus[Today, 1]', { expected: '...' }).done(),
    feature('Entity', 'knowledge').planned().effectful().gap('entity.country', 'Entity["Country", "Spain"]', { expected: '...' }).done(),
    feature('GeoPosition', 'geo').planned().pure().gap('geoposition.origin', 'GeoPosition[{0, 0}]', { expected: '...' }).done(),
    feature('UnitConvert', 'units')
        .unsupported()
        .pure()
        .gap('unitconvert.m_cm', 'UnitConvert[Quantity[1, "m"], "cm"]', { expected: 'Quantity[100, "cm"]' })
        .done(),
    feature('FileIO', 'io').planned().effectful().gap('file.absolute', 'AbsoluteFileName["."]', { expected: '...' }).done(),
    feature('Network', 'io').planned().effectful().gap('network.hostlookup', 'HostLookup["localhost"]', { expected: '...' }).done(),
);
