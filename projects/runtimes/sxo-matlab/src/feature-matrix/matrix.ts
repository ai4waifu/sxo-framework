import { feature, matrix } from '@sxo/harness';

/**
 * MATLAB dialect capability matrix.
 *
 * Status is honest against current SXO + Athena behavior.
 * Authored with `@sxo/harness` builders (not raw object literals).
 */
export const featureMatrix = matrix(
    feature('plus', 'arithmetic').supported().pure().eval('plus.basic', '2 + 3', '5').done(),
    feature('mtimes', 'arithmetic')
        .partial('scalar * and numeric nested-list matmul work; symbolic matrix * stays Times')
        .pure()
        .eval('mtimes.scalar', '2 * 3', '6')
        .gap('mtimes.2x2', '[1, 2; 3, 4]*[5, 6; 7, 8]', { expected: '[19, 22; 43, 50]', notes: 'currently [1, 2; 3, 4]*[5, 6; 7, 8]' })
        .done(),
    feature('times', 'arithmetic')
        .supported()
        .pure()
        .notes('.* → DotTimes elementwise')
        .eval('times.scalar', '2 .* [1, 2]', '[2, 4]')
        .eval('times.vec', '[1, 2].*[3, 4]', '[3, 8]')
        .done(),
    feature('power', 'arithmetic')
        .partial('scalar ^ and .^ OK; (x+1)^2 still expands')
        .pure()
        .eval('power.basic', '2^3', '8')
        .eval('power.elementwise', '[1, 2].^[2, 3]', '[1, 8]')
        .gap('power.binomsq', '(x + 1)^2', { expected: '(x + 1)^2', notes: 'currently expands' })
        .eval('power.vec_pow0', '[1, 2, 3].^0', '[1, 1, 1]')
        .done(),
    feature('mrdivide', 'arithmetic').supported().pure().eval('mrdivide.basic', '6 / 2', '3').done(),
    feature('rdivide', 'arithmetic')
        .supported()
        .pure()
        .notes('./ → DotDivide')
        .eval('rdivide.scalar', '1./2', '0.5')
        .eval('rdivide.vec', '[6, 8]./[2, 4]', '[3, 2]')
        .done(),
    feature('matrix', 'matrix')
        .partial('literal nested List + constructors via MatrixValue bridge; MatrixId IR pending')
        .pure()
        .eval('matrix.literal', '[1, 2; 3, 4]', '[1, 2; 3, 4]')
        .roundtrip('matrix.roundtrip', '[1 2; 3 4]', '[1, 2; 3, 4]')
        .done(),
    feature('colon', 'indexing')
        .supported()
        .pure()
        .notes('1:n and a:step:b expand to numeric row vectors')
        .eval('colon.range', '1:3', '[1, 2, 3]')
        .eval('colon.step', '1:2:10', '[1, 3, 5, 7, 9]')
        .done(),
    feature('subsref', 'indexing')
        .supported()
        .pure()
        .eval('subsref.vec', '[1, 2, 3](2)', '2')
        .eval('subsref.matrix', '[1, 2; 3, 4](1, 2)', '2')
        .eval('subsref.slice', '[1, 2, 3](1:2)', '[1, 2]')
        .done(),
    feature('end', 'indexing').supported().pure().eval('end.index', '[1, 2, 3](end)', '3').done(),
    feature('transpose', 'matrix')
        .unsupported("oak rejects ' transpose literal")
        .pure()
        .gap('transpose.vec', "[1; 2]'", { expected: '[1, 2]' })
        .done(),
    feature('ctranspose', 'matrix')
        .unsupported("depends on complex literal + '")
        .pure()
        .gap('ctranspose.basic', "[1+1i]'", { expected: '1-1i' })
        .done(),
    feature('assignment', 'session')
        .supported()
        .stateful()
        .notes('compound assignment binds in one evaluate string')
        .eval('assign.compound', 'x = 5; x + 1', '6')
        .eval('assign.persist', 'x = 5', '5', { notes: 'follow-up x+1 on same Session → 6 (napi session test)' })
        .done(),
    feature('sequence', 'session').supported().pure().eval('seq.last', '1; 2 + 2', '4').done(),
    feature('if', 'control').supported().pure().eval('if.else', 'if 1, 2, else, 3, end', '2').done(),
    feature('for', 'control')
        .supported()
        .stateful()
        .notes('for i=1:n last value and compound accumulator via shared Session bindings')
        .eval('for.last', 'for i=1:3, i, end', '3')
        .eval('for.sum', 's=0; for i=1:3, s=s+i; end; s', '6')
        .done(),
    feature('while', 'control')
        .supported()
        .stateful()
        .notes('while 0 skips body; empty result renders as []')
        .eval('while.false', 'while 0, 1, end', '[]')
        .done(),
    feature('function_handle', 'function')
        .unsupported('oak error on @(x); feval(@sin,0) strips @')
        .pure()
        .gap('fh.basic', 'f=@(x)x^2; f(4)', { expected: '16' })
        .gap('fh.feval', 'feval(@sin, 0)', { expected: '0', notes: 'currently feval(sin, 0)' })
        .done(),
    feature('eq', 'comparison').supported().pure().eval('eq.true', '3 == 3', 'true').done(),
    feature('ne', 'comparison')
        .partial('scalar ~= OK; vector [1,2]~=[1,3] stays Unequal head not elementwise mask')
        .pure()
        .eval('ne.true', '3 ~= 2', 'true')
        .eval('ne.false', '1 ~= 1', 'false')
        .gap('ne.vec', '[1, 2] ~= [1, 3]', { expected: '[0, 1]', notes: 'currently Unequal([1, 2], [1, 3])' })
        .done(),
    feature('le', 'comparison')
        .partial('scalar <= OK; vector / functional le(…) unevaluated')
        .pure()
        .eval('le.true', '2 <= 3', 'true')
        .gap('le.vec', '[1, 2] <= [1, 3]', { expected: '[1, 1]' })
        .gap('le.fn', 'le(1, 2)', { expected: 'true' })
        .done(),
    feature('ge', 'comparison')
        .partial('scalar >= OK; vector / functional ge(…) unevaluated')
        .pure()
        .eval('ge.true', '1 >= 1', 'true')
        .eval('ge.false', '1 >= 2', 'false')
        .gap('ge.vec', '[1, 2, 3] >= 2', { expected: '[0, 1, 1]' })
        .gap('ge.fn', 'ge(2, 2)', { expected: 'true' })
        .done(),
    feature('gt', 'comparison').supported().pure().eval('gt.true', '3 > 2', 'true').done(),
    feature('sin', 'elementary').supported().pure().eval('sin.0', 'sin(0)', '0').done(),
    feature('cos', 'elementary').supported().pure().eval('cos.0', 'cos(0)', '1').done(),
    feature('sqrt', 'elementary').supported().pure().eval('sqrt.4', 'sqrt(4)', '2').done(),
    feature('abs', 'elementary').supported().pure().eval('abs.neg', 'abs(-3)', '3').done(),
    feature('exp', 'elementary').supported().pure().eval('exp.0', 'exp(0)', '1').done(),
    feature('log', 'elementary').supported().pure().eval('log.1', 'log(1)', '0').done(),
    feature('listable_sin', 'elementary')
        .unsupported('sin on vector left as sin([...])')
        .pure()
        .gap('sin.listable', 'sin([0, pi/2])', { expected: '[0, 1]' })
        .done(),
    feature('simplify', 'simplify').supported().pure().eval('simplify.trig', 'sin(x)^2 + cos(x)^2', '1').done(),
    feature('diff', 'calculus')
        .partial('first derivative works; higher-order diff(f,x,2) unevaluated')
        .pure()
        .eval('diff.poly', 'diff(x^3, x)', '3*x^2')
        .eval('diff.sin', 'diff(sin(x), x)', 'cos(x)')
        .gap('diff.order2', 'diff(x^2, x, 2)', { expected: '2' })
        .done(),
    feature('int', 'calculus')
        .supported()
        .pure()
        .eval('int.poly', 'int(x^2, x)', '1/3*x^3')
        .eval('int.sin', 'int(sin(x), x)', '-cos(x)')
        .done(),
    feature('solve', 'solve')
        .planned('must lower to Athena SolveGoal')
        .pure()
        .gap('solve.linear', 'solve(x-1==0, x)', { expected: '1' })
        .done(),
    feature('mldivide', 'solve')
        .supported()
        .pure()
        .notes('exact numeric nested-list A\\b; symbolic stays unevaluated')
        .eval('mldivide.2x2', '[1,2;3,4] \\ [5;6]', '[-4; 9/2]')
        .done(),
    feature('linsolve', 'solve')
        .unsupported('linsolve → LinearSolve; same exact bridge as mldivide')
        .pure()
        .gap('linsolve.2x2', 'linsolve([1, 2; 3, 4], [5; 6])', { expected: '[-4; 9/2]', notes: 'currently linsolve([1, 2; 3, 4], [5; 6])' })
        .done(),
    feature('roots', 'solve').unsupported().pure().gap('roots.quad', 'roots([1, 0, -1])', { expected: '[1; -1]' }).done(),
    feature('det', 'linear_algebra').supported().pure().eval('det.2x2', 'det([1, 2; 3, 4])', '-2').done(),
    feature('inv', 'linear_algebra').unsupported().pure().gap('inv.diag', 'inv([1, 0; 0, 2])', { expected: '[1, 0; 0, 0.5]' }).done(),
    feature('rank', 'linear_algebra').unsupported().pure().gap('rank.def', 'rank([1, 2; 2, 4])', { expected: '1' }).done(),
    feature('eig', 'linear_algebra').unsupported().pure().gap('eig.sym', 'eig([1, 2; 2, 1])', { expected: '[3; -1]' }).done(),
    feature('eye', 'matrix').supported().pure().eval('eye.2', 'eye(2)', '[1, 0; 0, 1]').done(),
    feature('zeros', 'matrix').supported().pure().eval('zeros.23', 'zeros(2, 3)', '[0, 0, 0; 0, 0, 0]').done(),
    feature('ones', 'matrix').supported().pure().eval('ones.2', 'ones(2)', '[1, 1; 1, 1]').done(),
    feature('size', 'matrix').supported().pure().eval('size.2x2', 'size([1, 2; 3, 4])', '[2, 2]').done(),
    feature('length', 'matrix').supported().pure().eval('length.vec', 'length([1, 2, 3])', '3').done(),
    feature('sum', 'matrix')
        .supported()
        .pure()
        .notes('vector → scalar; matrix → column sums')
        .eval('sum.vec', 'sum([1, 2, 3])', '6')
        .eval('sum.matrix', 'sum([1, 2; 3, 4])', '[4, 6]')
        .done(),
    feature('max', 'matrix').unsupported().pure().gap('max.vec', 'max([1, 3, 2])', { expected: '3' }).done(),
    feature('plot', 'plot')
        .partial(
            'SVG→PNG visual: curve+L-axes readable; missing tick labels, no boxed frame, not MATLAB default blue. Negative a/b via unary-minus fold; 2-arg plot(f,[a,b]) gap',
        )
        .effectful()
        .plot('plot.square', 'plot(x^2, x, 0, 1)', { expected: '<svg' })
        .plot('plot.sin', 'plot(sin(x), x, 0, 6)', { expected: '<svg' })
        .gap('plot.neg_domain', 'plot(x^2, x, -1, 1)', { expected: '<svg', notes: 'currently not a supported 1-D plot form' })
        .gap('plot.range_vec', 'plot(sin(x), [-pi, pi])', { expected: '<svg', notes: 'surface sugar for domain vector' })
        .done(),
    feature('mesh', 'plot').unsupported().effectful().gap('mesh.peaks', 'mesh(peaks)', { expected: '<svg' }).done(),
    feature('complex', 'numeric')
        .unsupported('oak bad literal on 2i')
        .pure()
        .gap('complex.i', '1+2i', { expected: '1+2i' })
        .gap('complex.real', 'real(1+2i)', { expected: '1' })
        .done(),
    feature('pi', 'constant').partial().pure().eval('pi.symbol', 'pi', 'pi').done(),
    feature('true', 'constant').partial().pure().eval('true.atom', 'true', 'true').done(),
    feature('false', 'constant').partial().pure().eval('false.atom', 'false', 'false').done(),
    feature('disp', 'io').unsupported().effectful().gap('disp.1', 'disp(1)', { expected: '1' }).done(),
    feature('linspace', 'matrix').unsupported().pure().gap('linspace.3', 'linspace(0, 1, 3)', { expected: '[0, 0.5, 1]' }).done(),
    feature('diag', 'linear_algebra').unsupported().pure().gap('diag.vec', 'diag([1, 2])', { expected: '[1, 0; 0, 2]' }).done(),
    feature('trace', 'linear_algebra').unsupported().pure().gap('trace.2x2', 'trace([1, 2; 3, 4])', { expected: '5' }).done(),
    feature('norm', 'linear_algebra').unsupported().pure().gap('norm.34', 'norm([3, 4])', { expected: '5' }).done(),
    feature('dot', 'linear_algebra').unsupported().pure().gap('dot.2', 'dot([1, 2], [3, 4])', { expected: '11' }).done(),
    feature('cross', 'linear_algebra').unsupported().pure().gap('cross.ijk', 'cross([1, 0, 0], [0, 1, 0])', { expected: '[0, 0, 1]' }).done(),
    feature('reshape', 'matrix').unsupported().pure().gap('reshape.22', 'reshape([1, 2, 3, 4], 2, 2)', { expected: '[1, 3; 2, 4]' }).done(),
    feature('sort', 'matrix').unsupported().pure().gap('sort.vec', 'sort([3, 1, 2])', { expected: '[1, 2, 3]' }).done(),
    feature('mean', 'stats').unsupported().pure().gap('mean.vec', 'mean([1, 2, 3])', { expected: '2' }).done(),
    feature('std', 'stats').unsupported().pure().gap('std.vec', 'std([1, 2, 3])', { expected: '1' }).done(),
    feature('fft', 'signal').unsupported().pure().gap('fft.4', 'fft([1, 2, 3, 4])', { expected: '...' }).done(),
    feature('polyval', 'polynomial').unsupported().pure().gap('polyval.quad', 'polyval([1, 0, -1], 2)', { expected: '3' }).done(),
    feature('elementwise_compare', 'comparison')
        .unsupported('vectorized > / == become Greater/Equal heads, not logical masks')
        .pure()
        .gap('gt.vec', '[1, 2, 3] > 2', { expected: '[0, 0, 1]', notes: 'currently Greater([1, 2, 3], 2)' })
        .done(),
    feature('cell', 'types')
        .unsupported('SILENT WRONG: {1,2} evaluates to 2 (brace not cell)')
        .pure()
        .gap('cell.literal', '{1, 2}', { expected: '{1, 2}', notes: 'currently returns 2' })
        .gap('cell.ctor', 'cell(2, 1)', { expected: '{[]; []}' })
        .done(),
    feature('struct', 'types').unsupported().pure().gap('struct.basic', "struct('a', 1)", { expected: "struct('a',1)" }).done(),
    feature('ode45', 'ode')
        .unsupported('SILENT WRONG: ode45(@(t,y)y,[0,1],1) → 1 (strips to last arg)')
        .pure()
        .gap('ode45.strip', 'ode45(@(t,y)y, [0, 1], 1)', { expected: '...', notes: 'currently returns 1' })
        .done(),
    feature('fzero', 'solve')
        .unsupported('@ handle stripped; args mangled to fzero(x, -2+x^2, 1)')
        .pure()
        .gap('fzero.sqrt2', 'fzero(@(x)x^2-2, 1)', { expected: '1.4142' })
        .done(),
    feature('integral', 'calculus')
        .unsupported('@ stripped to integral(x, sin(x), 0, pi)')
        .pure()
        .gap('integral.sin', 'integral(@(x)sin(x), 0, pi)', { expected: '2' })
        .done(),
    feature('switch', 'control')
        .unsupported('SILENT WRONG: switch 1, case 1, 2, otherwise, 3, end → 3')
        .pure()
        .gap('switch.case1', 'switch 1, case 1, 2, otherwise, 3, end', { expected: '2', notes: 'currently returns 3' })
        .done(),
    feature('try_catch', 'control')
        .supported()
        .pure()
        .notes('oak Statement::Try → Athena Try[body, catch]; success and error paths')
        .eval('try.catch', "try, error('e'), catch, 1, end", '1')
        .eval('try.no_error', 'try, 2, catch, 3, end', '2')
        .done(),
    feature('global', 'session').unsupported('SILENT WRONG: global x → x').stateful().gap('global.strip', 'global x', { expected: '' }).done(),
    feature('persistent', 'session')
        .unsupported('SILENT WRONG: persistent y → y')
        .stateful()
        .gap('persistent.strip', 'persistent y', { expected: '' })
        .done(),
    feature('which', 'meta').unsupported('SILENT WRONG: which sin → sin').pure().gap('which.sin', 'which sin', { expected: '...' }).done(),
    feature('row_colon', 'indexing')
        .supported()
        .pure()
        .notes('All-colon row/col on literals; column as flat list of picks')
        .eval('row.colon', '[1, 2; 3, 4](1,:)', '[1, 2]')
        .eval('col.colon', '[1, 2; 3, 4](:,2)', '[2, 4]')
        .done(),
    feature('logical_index', 'indexing').unsupported().pure().gap('logical.gt', 'A=[1,2,3]; A(A>1)', { expected: '[2, 3]' }).done(),
    feature('surf', 'plot').unsupported().effectful().gap('surf.peaks', 'surf(peaks)', { expected: '<svg' }).done(),
    feature('contour', 'plot').unsupported().effectful().gap('contour.peaks', 'contour(peaks)', { expected: '<svg' }).done(),
    feature('figure', 'plot').unsupported().effectful().gap('figure.basic', 'figure', { expected: '...' }).done(),
    feature('sparse', 'matrix').unsupported().pure().gap('sparse.diag', 'sparse([1, 0; 0, 2])', { expected: '...' }).done(),
    feature('num2str', 'string').unsupported().pure().gap('num2str.3', 'num2str(3)', { expected: "'3'" }).done(),
    feature('str2num', 'string').unsupported().pure().gap('str2num.3', "str2num('3')", { expected: '3' }).done(),
    feature('pascal', 'matrix').unsupported().pure().gap('pascal.3', 'pascal(3)', { expected: '[1,1,1; 1,2,3; 1,3,6]' }).done(),
    feature('magic', 'matrix').unsupported().pure().gap('magic.3', 'magic(3)', { expected: '...' }).done(),
    feature('cond', 'linear_algebra').unsupported().pure().gap('cond.2x2', 'cond([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('null', 'linear_algebra').unsupported().pure().gap('null.rank1', 'null([1, 2; 2, 4])', { expected: '...' }).done(),
    feature('pinv', 'linear_algebra').unsupported().pure().gap('pinv.2x2', 'pinv([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('svd', 'linear_algebra').unsupported().pure().gap('svd.2x2', 'svd([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('kron', 'linear_algebra').unsupported().pure().gap('kron.basic', 'kron([1, 2], [3, 4])', { expected: '[3,4,6,8]' }).done(),
    feature('polyfit', 'fit').unsupported().pure().gap('polyfit.quad', 'polyfit([1, 2, 3], [1, 4, 9], 2)', { expected: '[1, 0, 0]' }).done(),
    feature('interp1', 'fit').unsupported().pure().gap('interp1.mid', 'interp1([0, 1], [0, 1], 0.5)', { expected: '0.5' }).done(),
    feature('ode23', 'ode')
        .unsupported('SILENT WRONG: ode23(@(t,y)y,[0,1],1) → 1 (same strip pattern as ode45)')
        .pure()
        .gap('ode23.strip', 'ode23(@(t,y)y, [0, 1], 1)', { expected: '...', notes: 'currently returns 1' })
        .done(),
    feature('fsolve', 'solve')
        .unsupported('@ handle stripped; args mangled like fzero')
        .pure()
        .gap('fsolve.sqrt2', 'fsolve(@(x)x^2-2, 1)', { expected: '1.4142' })
        .done(),
    feature('class', 'types').unsupported().pure().gap('class.double', 'class(1)', { expected: "'double'" }).done(),
    feature('isa', 'types').unsupported().pure().gap('isa.double', "isa(1, 'double')", { expected: '1' }).done(),
    feature('isnumeric', 'types').unsupported().pure().gap('isnumeric.1', 'isnumeric(1)', { expected: '1' }).done(),
    feature('assert', 'control').unsupported().pure().gap('assert.true', 'assert(1)', { expected: '...' }).done(),
    feature('imag_unit_literal', 'numeric')
        .unsupported('oak bad literal on bare 1i (also 2i in complex entry)')
        .pure()
        .gap('imag.1i', '1i', { expected: '1i' })
        .done(),
    feature('complex_ctor', 'numeric').unsupported().pure().gap('complex.ctor', 'complex(1, 2)', { expected: '1+2i' }).done(),
    feature('single', 'numeric').unsupported().pure().gap('single.1', 'single(1)', { expected: '1' }).done(),
    feature('nan', 'constant')
        .partial('lowercase atom retained; NaN arithmetic / isnan contract incomplete')
        .pure()
        .eval('nan.lower', 'nan', 'nan')
        .done(),
    feature('eps', 'constant').unsupported().pure().gap('eps.atom', 'eps', { expected: '...' }).done(),
    feature('diff_vector', 'array')
        .unsupported('vector difference op distinct from symbolic diff(f,x)')
        .pure()
        .gap('diffvec.3', 'diff([1, 4, 9])', { expected: '[3, 5]' })
        .done(),
    feature('cumsum', 'array').unsupported().pure().gap('cumsum.3', 'cumsum([1, 2, 3])', { expected: '[1, 3, 6]' }).done(),
    feature('contains', 'string').unsupported().pure().gap('contains.b', "contains('abc', 'b')", { expected: '1' }).done(),
    feature('string', 'string').unsupported().pure().gap('string.ctor', "string('ab')", { expected: '"ab"' }).done(),
    feature('jsonencode', 'io').unsupported().pure().gap('jsonencode.struct', "jsonencode(struct('a', 1))", { expected: '{"a":1}' }).done(),
    feature('datetime', 'types').unsupported().pure().gap('datetime.ymd', 'datetime(2020, 1, 1)', { expected: '...' }).done(),
    feature('readmatrix', 'io')
        .unsupported()
        .effectful()
        .gap('readmatrix.csv', "readmatrix('x.csv')", { expected: 'UnsupportedOperation' })
        .done(),
    feature('parfor', 'control')
        .unsupported('SILENT WRONG: parfor i=1:2, i, end → i (same strip as for)')
        .stateful()
        .gap('parfor.strip', 'parfor i=1:2, i, end', { expected: '2', notes: 'currently returns i' })
        .done(),
    feature('bsxfun', 'array')
        .unsupported('SILENT WRONG: @ stripped — bsxfun(@plus,…) → bsxfun(plus,…)')
        .pure()
        .gap('bsxfun.plus', 'bsxfun(@plus, [1, 2], [3; 4])', { expected: '[4, 5; 5, 6]', notes: 'currently bsxfun(plus, …)' })
        .done(),
    feature('arrayfun', 'array')
        .unsupported('@ stripped to bare sin')
        .pure()
        .gap('arrayfun.sin', 'arrayfun(@sin, [0, pi/2])', { expected: '[0, 1]' })
        .done(),
    feature('cellfun', 'array')
        .unsupported('SILENT WRONG: @ and cell both strip — cellfun(@numel,{1,2}) → cellfun(numel, 1, 2)')
        .pure()
        .gap('cellfun.numel', 'cellfun(@numel, {1, 2})', { expected: '[1, 1]' })
        .done(),
    feature('hold_on', 'plot')
        .unsupported('SILENT WRONG: hold on → on (command keyword stripped)')
        .effectful()
        .gap('hold.on', 'hold on', { expected: '...', notes: 'currently returns on' })
        .done(),
    feature('grid_on', 'plot')
        .unsupported('SILENT WRONG: grid on → on')
        .effectful()
        .gap('grid.on', 'grid on', { expected: '...', notes: 'currently returns on' })
        .done(),
    feature('axis', 'plot')
        .unsupported('SILENT WRONG: axis equal → equal')
        .effectful()
        .gap('axis.equal', 'axis equal', { expected: '...', notes: 'currently returns equal' })
        .done(),
    feature('profile', 'meta')
        .unsupported('SILENT WRONG: profile on → on')
        .effectful()
        .gap('profile.on', 'profile on', { expected: '...', notes: 'currently returns on' })
        .done(),
    feature('dbstop', 'meta')
        .unsupported('SILENT WRONG: dbstop if error → error')
        .effectful()
        .gap('dbstop.if_error', 'dbstop if error', { expected: '...', notes: 'currently returns error' })
        .done(),
    feature('ode15s', 'ode')
        .unsupported('SILENT WRONG: ode15s(@(t,y)y,[0,1],1) → 1')
        .pure()
        .gap('ode15s.strip', 'ode15s(@(t,y)y, [0, 1], 1)', { expected: '...', notes: 'currently returns 1' })
        .done(),
    feature('ode113', 'ode')
        .unsupported('SILENT WRONG: same last-arg strip as ode45')
        .pure()
        .gap('ode113.strip', 'ode113(@(t,y)y, [0, 1], 1)', { expected: '...', notes: 'currently returns 1' })
        .done(),
    feature('dde23', 'ode')
        .unsupported('SILENT WRONG: dde23(…) → [0, 2] (last arg)')
        .pure()
        .gap('dde23.strip', 'dde23(@(t,y,z)z, [1], 1, [0, 2])', { expected: '...', notes: 'currently returns [0, 2]' })
        .done(),
    feature('fminsearch', 'solve')
        .unsupported('@ handle stripped like fzero')
        .pure()
        .gap('fminsearch.x2', 'fminsearch(@(x)x^2, 1)', { expected: '0' })
        .done(),
    feature('quadgk', 'calculus')
        .unsupported('@ stripped to quadgk(x, sin(x), 0, pi)')
        .pure()
        .gap('quadgk.sin', 'quadgk(@(x)sin(x), 0, pi)', { expected: '2' })
        .done(),
    feature('containers_Map', 'types')
        .unsupported('SILENT WRONG: containers.Map → Map (package path stripped)')
        .pure()
        .gap('containers.map', 'containers.Map', { expected: 'containers.Map', notes: 'currently returns Map' })
        .done(),
    feature('py_list', 'interop')
        .unsupported('SILENT WRONG: py.list([1,2]) → list([1, 2])')
        .pure()
        .gap('py.list', 'py.list([1, 2])', { expected: '...', notes: 'currently list([1, 2])' })
        .done(),
    feature('classdef', 'oop')
        .unsupported('SILENT WRONG: classdef Foo, end → Foo')
        .stateful()
        .gap('classdef.foo', 'classdef Foo, end', { expected: '...', notes: 'currently returns Foo' })
        .done(),
    feature('spmd', 'parallel')
        .unsupported('SILENT WRONG: spmd, 1, end → 1')
        .stateful()
        .gap('spmd.strip', 'spmd, 1, end', { expected: '...', notes: 'currently returns 1' })
        .done(),
    feature('iscell', 'types')
        .unsupported('SILENT WRONG: iscell({1}) → iscell(1) (brace cell stripped first)')
        .pure()
        .gap('iscell.brace', 'iscell({1})', { expected: '1', notes: 'currently iscell(1)' })
        .done(),
    feature('all', 'array').unsupported().pure().gap('all.true', 'all([1, 1])', { expected: '1' }).done(),
    feature('any', 'array').unsupported().pure().gap('any.mixed', 'any([0, 1])', { expected: '1' }).done(),
    feature('find', 'array').unsupported().pure().gap('find.mask', 'find([0, 1, 0])', { expected: '2' }).done(),
    feature('unique', 'array').unsupported().pure().gap('unique.112', 'unique([1, 1, 2])', { expected: '[1, 2]' }).done(),
    feature('repmat', 'array').unsupported().pure().gap('repmat.21', 'repmat([1, 2], 2, 1)', { expected: '[1, 2; 1, 2]' }).done(),
    feature('tril', 'matrix').unsupported().pure().gap('tril.2x2', 'tril([1, 2; 3, 4])', { expected: '[1, 0; 3, 4]' }).done(),
    feature('qr', 'linear_algebra').unsupported().pure().gap('qr.2x2', 'qr([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('lu', 'linear_algebra').unsupported().pure().gap('lu.2x2', 'lu([1, 2; 3, 4])', { expected: '...' }).done(),
    feature('chol', 'linear_algebra').unsupported().pure().gap('chol.spd', 'chol([2, 1; 1, 2])', { expected: '...' }).done(),
    feature('expm', 'linear_algebra').unsupported().pure().gap('expm.rot', 'expm([0, 1; -1, 0])', { expected: '...' }).done(),
    feature('floor', 'numeric').unsupported().pure().gap('floor.2_7', 'floor(2.7)', { expected: '2' }).done(),
    feature('mod', 'numeric').unsupported().pure().gap('mod.10_3', 'mod(10, 3)', { expected: '1' }).done(),
    feature('hypot', 'numeric').unsupported().pure().gap('hypot.34', 'hypot(3, 4)', { expected: '5' }).done(),
    feature('inf', 'constant').partial('symbol retained; no Inf arithmetic contract yet').pure().eval('inf.atom', 'inf', 'inf').done(),
    feature('NaN', 'constant').partial().pure().eval('NaN.capital', 'NaN', 'NaN').done(),
    feature('rand', 'random').unsupported().effectful().gap('rand.2', 'rand(2)', { expected: '...' }).done(),
    feature('hilb', 'matrix').unsupported().pure().gap('hilb.3', 'hilb(3)', { expected: '...' }).done(),
    feature('sym', 'symbolic').unsupported().pure().gap('sym.x', "sym('x')", { expected: 'x' }).done(),
    feature('vpa', 'symbolic').unsupported().pure().gap('vpa.pi', 'vpa(pi, 10)', { expected: '3.141592654' }).done(),
    feature('jsondecode', 'io').unsupported().pure().gap('jsondecode.obj', 'jsondecode(\'{"a":1}\')', { expected: '...' }).done(),
    feature('sprintf', 'string').unsupported().pure().gap('sprintf.d', "sprintf('%d', 1)", { expected: "'1'" }).done(),
    feature('legend', 'plot').unsupported().effectful().gap('legend.a', "legend('a')", { expected: '...' }).done(),
    feature('subplot', 'plot').unsupported().effectful().gap('subplot.121', 'subplot(1, 2, 1)', { expected: '...' }).done(),
    feature('and', 'logic').supported().pure().notes('numeric short-circuit style && on 0/1').eval('and.10', '1 && 0', 'false').done(),
    feature('or', 'logic').supported().pure().eval('or.10', '1 || 0', 'true').done(),
    feature('not', 'logic').supported().pure().eval('not.1', '~1', 'false').eval('not.0', '~0', 'true').done(),
    feature('xor', 'logic').unsupported().pure().gap('xor.10', 'xor(1, 0)', { expected: '1' }).done(),
    feature('bitand', 'bitwise').unsupported().pure().gap('bitand.63', 'bitand(6, 3)', { expected: '2' }).done(),
    feature('subsasgn', 'session')
        .unsupported(
            'SILENT WRONG: indexed / grow assignment does not persist; A=[]; A(1)=1; A → A; A(3,3)=1 after zeros → A; end+1 / end-1 often oak error',
        )
        .stateful()
        .gap('subsasgn.vec', 'A=[1, 2, 3]; A(2)=9; A', { expected: '[1, 9, 3]', notes: 'currently returns A' })
        .gap('subsasgn.grow', 'A=zeros(2); A(3, 3)=1; A', { expected: '...', notes: 'currently returns A' })
        .gap('subsasgn.end_plus', 'B=1:4; B(end+1)=5', { expected: '[1, 2, 3, 4, 5]', notes: 'oak error node' })
        .done(),
    feature('deal', 'session')
        .unsupported('multi-assign [a,b]=deal(1,2) does not bind; [~,b]=max(...) oak error')
        .pure()
        .gap('deal.multi', '[a, b]=deal(1, 2)', { expected: '...', notes: 'currently returns deal(1, 2)' })
        .done(),
    feature('strcat', 'string').unsupported().pure().gap('strcat.ab', "strcat('a', 'b')", { expected: "'ab'" }).done(),
    feature('strcmp', 'string').unsupported().pure().gap('strcmp.eq', "strcmp('a', 'a')", { expected: '1' }).done(),
    feature('strjoin', 'string')
        .unsupported("SILENT WRONG: cell brace stripped — strjoin({'a','b'},',') → strjoin('a', 'b', ',')")
        .pure()
        .gap('strjoin.ab', "strjoin({'a', 'b'}, ',')", { expected: "'a,b'" })
        .done(),
    feature('missing', 'types')
        .partial('atom retained; ismissing/rmmissing unevaluated')
        .pure()
        .eval('missing.atom', 'missing', 'missing')
        .done(),
    feature('ismissing', 'types').unsupported().pure().gap('ismissing.missing', 'ismissing(missing)', { expected: '1' }).done(),
    feature('lsqcurvefit', 'solve')
        .unsupported('SILENT WRONG: @ stripped and result collapses to last arg [1,2]')
        .pure()
        .gap('lsqcurvefit.strip', 'lsqcurvefit(@(x,xdata)x*xdata, 1, [1, 2], [1, 2])', { expected: '1', notes: 'currently returns [1, 2]' })
        .done(),
    feature('fminbnd', 'solve').unsupported('@ handle stripped').pure().gap('fminbnd.x2', 'fminbnd(@(x)x^2, -1, 1)', { expected: '0' }).done(),
    feature('syms', 'symbolic')
        .unsupported('SILENT WRONG: syms x → x (declaration stripped like global)')
        .stateful()
        .gap('syms.strip', 'syms x', { expected: '...', notes: 'currently returns x' })
        .done(),
    feature('expand', 'symbolic')
        .unsupported('args already wrongly powered: expand((x+1)^2) sees expand(1+x^2)')
        .pure()
        .gap('expand.binomsq', 'expand((x + 1)^2)', { expected: 'x^2 + 2*x + 1' })
        .done(),
    feature('limit', 'symbolic').unsupported().pure().gap('limit.sinc', 'limit(sin(x)/x, x, 0)', { expected: '1' }).done(),
    feature('dsolve', 'symbolic').planned().pure().gap('dsolve.exp', 'dsolve(diff(y)==y)', { expected: 'C1*exp(t)' }).done(),
    feature('eval', 'meta').unsupported().effectful().gap('eval.plus', "eval('1+1')", { expected: '2' }).done(),
    feature('feval', 'meta').unsupported().pure().gap('feval.sin', "feval('sin', 0)", { expected: '0' }).done(),
    feature('func2str', 'meta')
        .unsupported('@ stripped: func2str(@sin) → func2str(sin)')
        .pure()
        .gap('func2str.sin', 'func2str(@sin)', { expected: "'sin'" })
        .done(),
    feature('pwd', 'io').unsupported().effectful().gap('pwd.basic', 'pwd', { expected: '...' }).done(),
    feature('exist', 'meta').unsupported().pure().gap('exist.sin', "exist('sin', 'builtin')", { expected: '5' }).done(),
    feature('fullfile', 'io').unsupported().pure().gap('fullfile.ab', "fullfile('a', 'b')", { expected: '...' }).done(),
    feature('py_math_sqrt', 'interop')
        .unsupported('SILENT WRONG: py.math.sqrt(4) and math.sqrt(4) both collapse to matlab sqrt → 2')
        .pure()
        .gap('py.math.sqrt', 'py.math.sqrt(4)', { expected: '...', notes: 'currently returns 2 via path strip to sqrt' })
        .done(),
    feature('uifigure', 'ui').unsupported().effectful().gap('uifigure.basic', 'uifigure', { expected: '...' }).done(),
    feature('msgbox', 'ui').unsupported().effectful().gap('msgbox.a', "msgbox('a')", { expected: '...' }).done(),
    feature('ieee_edge', 'numeric')
        .partial('SILENT WRONG: 0/0→0 and Inf-Inf→0 (MATLAB expects NaN); 0^0→1 matches MATLAB')
        .pure()
        .gap('ieee.0over0', '0/0', { expected: 'NaN', notes: 'currently 0' })
        .gap('ieee.inf_minus_inf', 'Inf - Inf', { expected: 'NaN', notes: 'currently 0' })
        .eval('ieee.0pow0', '0^0', '1', { notes: 'MATLAB-compatible' })
        .done(),
    feature('bitor_op', 'logic')
        .unsupported('SILENT WRONG: 1|0 → 0 (expect 1); [1,0]|[0,1] → [0,1] (expect [1,1])')
        .pure()
        .gap('bitor.scalar', '1 | 0', { expected: '1', notes: 'currently 0' })
        .gap('bitor.vec', '[1, 0] | [0, 1]', { expected: '[1, 1]', notes: 'currently [0, 1]' })
        .done(),
    feature('bitand_op', 'logic')
        .partial('scalar 1&0 → false OK; vector [1,0]&[1,1] → [1,1] SILENT WRONG (expect [1,0])')
        .pure()
        .eval('bitand.scalar', '1 & 0', 'false')
        .gap('bitand.vec', '[1, 0] & [1, 1]', { expected: '[1, 0]', notes: 'currently [1, 1]' })
        .done(),
    feature('colon_all', 'indexing')
        .unsupported('SILENT WRONG: A(:) → A()')
        .pure()
        .gap('colon.all', 'A=[1, 2; 3, 4]; A(:)', { expected: '[1; 3; 2; 4]', notes: 'currently A()' })
        .done(),
    feature('end_minus', 'indexing')
        .unsupported('oak error on A(end-1) and A(1:2:end)')
        .pure()
        .gap('end.minus1', 'A=[1, 2, 3]; A(end-1)', { expected: '2' })
        .done(),
    feature('plus_eq', 'session')
        .unsupported('oak error on x+=1 and A(1)+=1')
        .stateful()
        .gap('pluseq.x', 'x=1; x+=1', { expected: '2' })
        .done(),
    feature('close_all', 'plot')
        .unsupported('SILENT WRONG: close all → all')
        .effectful()
        .gap('close.all', 'close all', { expected: '...', notes: 'currently returns all' })
        .done(),
    feature('hold_off', 'plot')
        .unsupported('SILENT WRONG: hold off → off')
        .effectful()
        .gap('hold.off', 'hold off', { expected: '...', notes: 'currently returns off' })
        .done(),
    feature('colormap', 'plot')
        .unsupported('SILENT WRONG: colormap jet → jet')
        .effectful()
        .gap('colormap.jet', 'colormap jet', { expected: '...', notes: 'currently returns jet' })
        .done(),
    feature('format', 'meta')
        .unsupported('SILENT WRONG: format long → long')
        .effectful()
        .gap('format.long', 'format long', { expected: '...', notes: 'currently returns long' })
        .done(),
    feature('simplify_trig', 'symbolic')
        .supported()
        .pure()
        .notes('same identity as simplify entry; explicit symbolic toolbox spelling')
        .eval('simplify_trig.pythag', 'simplify(sin(x)^2 + cos(x)^2)', '1')
        .done(),
    feature('isequal', 'comparison').unsupported().pure().gap('isequal.vec', 'isequal([1, 2], [1, 2])', { expected: '1' }).done(),
    feature('lt_chain', 'comparison')
        .partial('scalar 1<2<3 via Athena compare chain; elementwise vector Less still open')
        .pure()
        .eval('ltchain.123', '1 < 2 < 3', 'true')
        .gap('ltchain.vec', '[1, 2, 3] < 2', { expected: '[1, 0, 0]' })
        .done(),
    feature('odeset', 'ode')
        .unsupported('SILENT WRONG: ode45(..., odeset(...)) collapses to odeset(...) last arg')
        .pure()
        .gap('odeset.strip', "ode45(@(t,y)y, [0, 1], 1, odeset('RelTol', 1e-3))", {
            expected: '...',
            notes: "currently returns odeset('RelTol', 0.001)",
        })
        .done(),
    feature('rref', 'linear_algebra').unsupported().pure().gap('rref.basic', 'rref([1, 2, 3; 4, 5, 6])', { expected: '...' }).done(),
    feature('conv', 'signal').unsupported().pure().gap('conv.basic', 'conv([1, 1], [1, -1])', { expected: '[1, 0, -1]' }).done(),
    feature('polyder', 'polynomial').unsupported().pure().gap('polyder.quad', 'polyder([1, 2, 1])', { expected: '[2, 2]' }).done(),
    feature('scatter', 'plot').unsupported().effectful().gap('scatter.basic', 'scatter([1, 2], [3, 4])', { expected: '<svg' }).done(),
    feature('bar', 'plot').unsupported().effectful().gap('bar.3', 'bar([1, 2, 3])', { expected: '<svg' }).done(),
    feature('fourier_sym', 'symbolic')
        .unsupported('SILENT WRONG: fourier(exp(-x^2)) → fourier(exp(x^2)) sign flip')
        .pure()
        .gap('fourier.gauss_sign', 'fourier(exp(-x^2))', { expected: '...', notes: 'currently fourier(exp(x^2))' })
        .done(),
    feature('methods_meta', 'meta')
        .unsupported("SILENT WRONG: methods('double') → 'double'")
        .pure()
        .gap('methods.double', "methods('double')", { expected: '...', notes: "currently returns 'double'" })
        .done(),
    feature('true_bitor', 'logic')
        .unsupported('SILENT WRONG: true | false → false (same bug as 1|0)')
        .pure()
        .gap('true.bitor', 'true | false', { expected: '1', notes: 'currently false' })
        .done(),
    feature('true_bitand', 'logic')
        .partial('true & false → false OK; true && false stays And(true,false) unevaluated')
        .pure()
        .eval('true.bitand', 'true & false', 'false')
        .gap('true.and_sc', 'true && false', { expected: '0', notes: 'currently And(true, false)' })
        .done(),
    feature('i_squared', 'numeric')
        .unsupported('bare i/j symbols retained; 1+2i / 1i still oak bad literal')
        .pure()
        .gap('i.sq', 'i^2', { expected: '-1' })
        .gap('j.sq', 'j^2', { expected: '-1' })
        .done(),
    feature('logical', 'types').unsupported().pure().gap('logical.vec', 'logical([1, 0, 2])', { expected: '[1, 0, 1]' }).done(),
    feature('intersect', 'array').unsupported().pure().gap('intersect.basic', 'intersect([1, 2, 3], [2, 3, 4])', { expected: '[2, 3]' }).done(),
    feature('union', 'array').unsupported().pure().gap('union.basic', 'union([1, 2], [2, 3])', { expected: '[1, 2, 3]' }).done(),
    feature('setdiff', 'array').unsupported().pure().gap('setdiff.basic', 'setdiff([1, 2, 3], [2])', { expected: '[1, 3]' }).done(),
    feature('ismember', 'array').unsupported().pure().gap('ismember.2', 'ismember([1, 2, 3], 2)', { expected: '[0, 1, 0]' }).done(),
    feature('ifft', 'signal').unsupported().pure().gap('ifft.roundtrip', 'ifft(fft([1, 2, 3, 4]))', { expected: '[1, 2, 3, 4]' }).done(),
    feature('filter', 'signal').unsupported().pure().gap('filter.basic', 'filter([1], [1, -0.5], ones(1, 5))', { expected: '...' }).done(),
    feature('conv2', 'signal').unsupported().pure().gap('conv2.basic', 'conv2([1, 2; 3, 4], [1, 1; 1, 1])', { expected: '...' }).done(),
    feature('blkdiag', 'matrix').unsupported().pure().gap('blkdiag.eye3', 'blkdiag(eye(2), 3)', { expected: '...' }).done(),
    feature('randperm', 'random').unsupported().effectful().gap('randperm.5', 'randperm(5)', { expected: '...' }).done(),
    feature('builtin', 'meta').unsupported().pure().gap('builtin.sin', "builtin('sin', 0)", { expected: '0' }).done(),
    feature('char', 'string').unsupported().pure().gap('char.65', 'char(65)', { expected: "'A'" }).done(),
    feature('semilogx', 'plot').unsupported().effectful().gap('semilogx.basic', 'semilogx(1:3, 1:3)', { expected: '<svg' }).done(),
    feature('loglog', 'plot').unsupported().effectful().gap('loglog.basic', 'loglog(1:3, 1:3)', { expected: '<svg' }).done(),
    feature('pie', 'plot').unsupported().effectful().gap('pie.3', 'pie([1, 2, 3])', { expected: '<svg' }).done(),
    feature('warndlg', 'ui').unsupported().effectful().gap('warndlg.w', "warndlg('w')", { expected: '...' }).done(),
    feature('latex_sym', 'symbolic').unsupported().pure().gap('latex.x2', "latex(sym('x^2'))", { expected: '...' }).done(),
    feature('preincrement', 'session')
        .unsupported('SILENT WRONG: ++A → A; A++ oak error')
        .stateful()
        .gap('preinc.A', '++A', { expected: '...', notes: 'currently returns A' })
        .done(),
    feature('gpuArray_zeros', 'interop')
        .unsupported('SILENT WRONG: gpuArray.zeros(2) → zeros(2) (package path stripped)')
        .pure()
        .gap('gpuarray.zeros', 'gpuArray.zeros(2)', { expected: '...', notes: 'currently zeros(2)' })
        .done(),
    feature('coder_typeof', 'interop')
        .unsupported('SILENT WRONG: coder.typeof(1) → typeof(1)')
        .pure()
        .gap('coder.typeof', 'coder.typeof(1)', { expected: '...', notes: 'currently typeof(1)' })
        .done(),
    feature('hex2dec', 'numeric').unsupported().pure().gap('hex2dec.ff', "hex2dec('FF')", { expected: '255' }).done(),
    feature('dec2hex', 'numeric').unsupported().pure().gap('dec2hex.255', 'dec2hex(255)', { expected: "'FF'" }).done(),
    feature('isempty', 'types')
        .unsupported()
        .pure()
        .gap('isempty.empty', 'isempty([])', { expected: '1' })
        .gap('isempty.zero', 'isempty(0)', { expected: '0' })
        .done(),
    feature('isscalar', 'types').unsupported().pure().gap('isscalar.1', 'isscalar(1)', { expected: '1' }).done(),
    feature('isvector', 'types').unsupported().pure().gap('isvector.row', 'isvector([1, 2])', { expected: '1' }).done(),
    feature('isrow', 'types').unsupported().pure().gap('isrow.12', 'isrow([1, 2])', { expected: '1' }).done(),
    feature('iscolumn', 'types').unsupported().pure().gap('iscolumn.12', 'iscolumn([1; 2])', { expected: '1' }).done(),
    feature('numel', 'matrix').unsupported().pure().gap('numel.empty', 'numel([])', { expected: '0' }).done(),
    feature('nan_matrix', 'matrix').unsupported().pure().gap('nan.2', 'nan(2)', { expected: '[NaN, NaN; NaN, NaN]' }).done(),
    feature('inf_matrix', 'matrix').unsupported().pure().gap('inf.2', 'inf(2)', { expected: '...' }).done(),
    feature('true_matrix', 'matrix').unsupported().pure().gap('true.23', 'true(2, 3)', { expected: '...' }).done(),
    feature('speye', 'matrix').unsupported().pure().gap('speye.3', 'speye(3)', { expected: '...' }).done(),
    feature('nnz', 'matrix').unsupported().pure().gap('nnz.speye2', 'nnz(speye(2))', { expected: '2' }).done(),
    feature('accumarray', 'array')
        .unsupported()
        .pure()
        .gap('accumarray.basic', 'accumarray([1; 2; 1], [10; 20; 30])', { expected: '[40; 20]' })
        .done(),
    feature('logspace', 'matrix').unsupported().pure().gap('logspace.3', 'logspace(0, 2, 3)', { expected: '[1, 10, 100]' }).done(),
    feature('normcdf', 'stats').unsupported().pure().gap('normcdf.0', 'normcdf(0)', { expected: '0.5' }).done(),
    feature('matlab_lang_on', 'interop')
        .unsupported('SILENT WRONG: matlab.lang.OnOffSwitchState.on → on')
        .pure()
        .gap('matlab.lang.on', 'matlab.lang.OnOffSwitchState.on', { expected: '...', notes: 'currently returns on' })
        .done(),
    feature('times_eq', 'session')
        .unsupported('oak error on A.*=3 / A./=2 / x^=2')
        .stateful()
        .gap('timeseq.elem', 'A=[1, 2]; A.*=3', { expected: '[3, 6]' })
        .done(),
    feature('log2', 'numeric').unsupported().pure().gap('log2.8', 'log2(8)', { expected: '3' }).done(),
    feature('pow2', 'numeric').unsupported().pure().gap('pow2.3', 'pow2(3)', { expected: '8' }).done(),
    feature('nextpow2', 'numeric').unsupported().pure().gap('nextpow2.5', 'nextpow2(5)', { expected: '3' }).done(),
    feature('flintmax', 'constant').unsupported().pure().gap('flintmax.atom', 'flintmax', { expected: '...' }).done(),
    feature('zeros_empty', 'matrix').unsupported().pure().gap('zeros.0x5', 'zeros(0, 5)', { expected: 'zeros(0,5)' }).done(),
    feature('ones_empty', 'matrix').unsupported().pure().gap('ones.5x0', 'ones(5, 0)', { expected: 'ones(5,0)' }).done(),
    feature('eye_empty', 'matrix').unsupported().pure().gap('eye.0', 'eye(0)', { expected: '[]' }).done(),
    feature('isdiag', 'matrix').unsupported().pure().gap('isdiag.eye', 'isdiag(eye(3))', { expected: '1' }).done(),
    feature('issymmetric', 'matrix').unsupported().pure().gap('issymmetric.eye', 'issymmetric(eye(3))', { expected: '1' }).done(),
    feature('istril', 'matrix').unsupported().pure().gap('istril.tril', 'istril(tril(ones(3)))', { expected: '1' }).done(),
    feature('bitcmp', 'bitwise').unsupported().pure().gap('bitcmp.u8', "bitcmp(1, 'uint8')", { expected: '254' }).done(),
    feature('fillmissing', 'types')
        .unsupported()
        .pure()
        .gap('fillmissing.const', "fillmissing([1, NaN], 'constant', 0)", { expected: '[1, 0]' })
        .done(),
    feature('odeset_opts', 'ode')
        .unsupported('odeset itself echoes; scientific 1e-6 becomes decimal')
        .pure()
        .gap('odeset.reltol', "odeset('RelTol', 1e-6)", { expected: '...' })
        .done(),
    feature('pcg', 'linear_algebra').unsupported().pure().gap('pcg.eye', 'pcg(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('eq_fn', 'comparison')
        .unsupported('functional eq([1,2],[1,2]) unevaluated; true==1 stays Equal head')
        .pure()
        .gap('eq.fn_vec', 'eq([1, 2], [1, 2])', { expected: '[1, 1]' })
        .gap('eq.true_num', 'true == 1', { expected: '1', notes: 'currently Equal(true, 1)' })
        .done(),
    feature('isequaln', 'comparison').unsupported().pure().gap('isequaln.nan', 'isequaln([NaN], [NaN])', { expected: '1' }).done(),
    feature('isnan', 'predicates')
        .unsupported()
        .pure()
        .gap('isnan.nan', 'isnan(NaN)', { expected: '1' })
        .gap('isnan.0', 'isnan(0)', { expected: '0' })
        .done(),
    feature('nan_eq', 'comparison')
        .unsupported('NaN==NaN / NaN~=NaN stay Equal/Unequal heads (IEEE unmet)')
        .pure()
        .gap('nan.eq', 'NaN == NaN', { expected: '0', notes: 'currently Equal(NaN, NaN)' })
        .gap('nan.ne', 'NaN ~= NaN', { expected: '1', notes: 'currently Unequal(NaN, NaN)' })
        .done(),
    feature('isstring', 'types')
        .unsupported()
        .pure()
        .gap('isstring.dq', 'isstring("a")', { expected: '1' })
        .gap('isstring.sq', "isstring('a')", { expected: '0' })
        .done(),
    feature('ischar', 'types')
        .unsupported()
        .pure()
        .gap('ischar.sq', "ischar('a')", { expected: '1' })
        .gap('ischar.dq', 'ischar("a")', { expected: '0' })
        .done(),
    feature('string_plus', 'string')
        .unsupported()
        .pure()
        .gap('string.plus', '"hello" + "world"', { expected: '"helloworld"' })
        .gap('string.append', 'append("a", "b")', { expected: '"ab"' })
        .done(),
    feature('startsWith', 'string').unsupported().pure().gap('startswith.a', 'startsWith("abc", "a")', { expected: '1' }).done(),
    feature('endsWith', 'string').unsupported().pure().gap('endswith.c', 'endsWith("abc", "c")', { expected: '1' }).done(),
    feature('erase', 'string').unsupported().pure().gap('erase.b', 'erase("abc", "b")', { expected: '"ac"' }).done(),
    feature('extractAfter', 'string').unsupported().pure().gap('extractafter.a', 'extractAfter("abc", "a")', { expected: '"bc"' }).done(),
    feature('matches', 'string').unsupported().pure().gap('matches.digits', 'matches("abc", digitsPattern)', { expected: '0' }).done(),
    feature('wildcardPattern', 'string').unsupported().pure().gap('wildcard.m', 'wildcardPattern("*.m")', { expected: '...' }).done(),
    feature('datetime_arith', 'datetime')
        .unsupported()
        .pure()
        .gap('datetime.caldays', 'datetime(2020, 1, 1) + caldays(1)', { expected: 'datetime(2020, 1, 2)' })
        .gap('datetime.days', 'datetime(2020, 1, 1) + days(1)', { expected: 'datetime(2020, 1, 2)' })
        .done(),
    feature('caldiff', 'datetime')
        .unsupported()
        .pure()
        .gap('caldiff.year', 'caldiff(datetime(2020, 1, 1), datetime(2021, 1, 1))', { expected: '...' })
        .done(),
    feature('outerjoin', 'table')
        .unsupported("SILENT WRONG: cell {'k'} in VariableNames stripped to 'k'")
        .pure()
        .gap('outerjoin.k', "outerjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", {
            expected: '...',
            notes: "currently VariableNames 'k' without cell",
        })
        .done(),
    feature('leftjoin', 'table')
        .unsupported('same cell VariableNames strip as outerjoin')
        .pure()
        .gap('leftjoin.k', "leftjoin(table([1; 2], 'VariableNames', {'k'}), table([2; 3], 'VariableNames', {'k'}))", { expected: '...' })
        .done(),
    feature('spalloc', 'matrix').unsupported().pure().gap('spalloc.332', 'spalloc(3, 3, 2)', { expected: '...' }).done(),
    feature('sparse_ijv', 'matrix').unsupported().pure().gap('sparse.ijv', 'sparse(1, 2, 3, 4, 4)', { expected: '...' }).done(),
    feature('full_speye', 'matrix').unsupported().pure().gap('full.speye2', 'full(speye(2))', { expected: '[1, 0; 0, 1]' }).done(),
    feature('spones', 'matrix').unsupported().pure().gap('spones.eye', 'spones(speye(2))', { expected: '...' }).done(),
    feature('spfun', 'matrix')
        .unsupported('SILENT WRONG: @ stripped — spfun(@sqrt,speye(2)) → spfun(sqrt, speye(2))')
        .pure()
        .gap('spfun.sqrt', 'spfun(@sqrt, speye(2))', { expected: '...', notes: 'currently spfun(sqrt, speye(2))' })
        .done(),
    feature('minres', 'linear_algebra').unsupported().pure().gap('minres.eye', 'minres(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('cgs', 'linear_algebra').unsupported().pure().gap('cgs.eye', 'cgs(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('lsqr', 'linear_algebra').unsupported().pure().gap('lsqr.eye', 'lsqr(speye(3), ones(3, 1))', { expected: '...' }).done(),
    feature('svd_econ', 'linear_algebra').unsupported().pure().gap('svd.econ', 'svd(magic(3), "econ")', { expected: '...' }).done(),
    feature('end_slice', 'indexing')
        .unsupported('D(1:end-1) / E(1:end,end) oak error; C(0) stays C(0)')
        .pure()
        .gap('end.slice_minus', 'D=[1, 2, 3]; D(1:end-1)', { expected: '[1, 2]', notes: 'oak error node' })
        .gap('end.slice_2d', 'E=[1, 2; 3, 4]; E(1:end, end)', { expected: '[2; 4]', notes: 'oak error node' })
        .done(),
    feature('ginput', 'plot').planned().effectful().gap('ginput.basic', 'ginput', { expected: '...' }).done(),
    feature('zoom', 'plot').planned().effectful().gap('zoom.basic', 'zoom', { expected: '...' }).done(),
    feature('uigridlayout', 'ui').planned().effectful().gap('uigridlayout.basic', 'uigridlayout', { expected: '...' }).done(),
    feature('drawnow', 'ui').planned().effectful().gap('drawnow.basic', 'drawnow', { expected: '...' }).done(),
    feature('compose', 'functional').unsupported().pure().gap('compose.sincos', 'compose(sin, cos, 0)', { expected: '0' }).done(),
);
