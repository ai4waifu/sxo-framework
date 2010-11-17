import { feature } from '@sxo/harness';

export const sessionFeatures = [
    feature('assignment', 'session')
        .supported()
        .stateful()
        .notes('compound assignment binds in one evaluate string; Own then subsref')
        .eval('assign.compound', 'x = 5; x + 1', '6')
        .eval('assign.persist', 'x = 5', '5', { notes: 'follow-up x+1 on same Session → 6 (napi session test)' })
        .eval('assign.then_index', 'A = [10, 20]; A(2)', '20')
        .eval('assign.then_linear', 'M = [1, 2; 3, 4]; M(2)', '3')
        .done(),
    feature('sequence', 'session').supported().pure().eval('seq.last', '1; 2 + 2', '4').done(),
    feature('global', 'session')
        .unsupported('parse keeps Global Form via oak Statement::Global; workspace declaration runtime still open')
        .stateful()
        .gap('global.strip', 'global x', { expected: '', notes: 'Form Global[x]; eval Reject (was silent → x)' })
        .done(),
    feature('persistent', 'session')
        .unsupported('parse keeps Persistent Form via oak Statement::Persistent; function-local persist runtime still open')
        .stateful()
        .gap('persistent.strip', 'persistent y', { expected: '', notes: 'Form Persistent[y]; eval Reject (was silent → y)' })
        .done(),
    feature('subsasgn', 'session')
        .partial('scalar writes plus column All assignment via Athena StoreIndex')
        .stateful()
        .eval('subsasgn.vec', 'A=[1, 2, 3]; A(2)=9; A', '[1, 9, 3]')
        .eval('subsasgn.mat', 'M=[1, 2; 3, 4]; M(1, 2)=9; M', '[1, 9; 3, 4]')
        .eval('subsasgn.col', 'M=[1, 2; 3, 4]; M(:, 2)=[9; 8]; M', '[1, 9; 3, 8]')
        .eval('subsasgn.grow', 'A=zeros(2); A(3, 3)=1; A', '[0, 0, 0; 0, 0, 0; 0, 0, 1]')
        .eval('subsasgn.end_plus', 'B=1:4; B(end+1)=5; B', '[1, 2, 3, 4, 5]')
        .done(),
    feature('deal', 'session')
        .unsupported('multi-assign [a,b]=deal(1,2) does not bind; [~,b]=max(...) oak error')
        .pure()
        .gap('deal.multi', '[a, b]=deal(1, 2)', { expected: '...', notes: 'currently returns deal(1, 2)' })
        .done(),
    feature('plus_eq', 'session')
        .unsupported('oak error on x+=1 and A(1)+=1')
        .stateful()
        .gap('pluseq.x', 'x=1; x+=1', { expected: '2' })
        .done(),
    feature('preincrement', 'session')
        .unsupported('MATLAB has no ++; ++A is double unary plus (= A). A++ is a syntax error')
        .stateful()
        .gap('preinc.A', '++A', {
            expected: 'A',
            notes: 'not C/Octave increment; unary + twice collapses to A (language-correct)',
        })
        .done(),
    feature('times_eq', 'session')
        .unsupported('oak error on A.*=3 / A./=2 / x^=2')
        .stateful()
        .gap('timeseq.elem', 'A=[1, 2]; A.*=3', { expected: '[3, 6]' })
        .done(),
];
