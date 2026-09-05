import { feature } from '@sxo/harness';

export const sessionFeatures = [
    feature('assignment', 'session')
        .supported()
        .stateful()
        .notes('compound assignment binds in one evaluate string')
        .eval('assign.compound', 'x = 5; x + 1', '6')
        .eval('assign.persist', 'x = 5', '5', { notes: 'follow-up x+1 on same Session → 6 (napi session test)' })
        .done(),
    feature('sequence', 'session').supported().pure().eval('seq.last', '1; 2 + 2', '4').done(),
    feature('global', 'session').unsupported('SILENT WRONG: global x → x').stateful().gap('global.strip', 'global x', { expected: '' }).done(),
    feature('persistent', 'session')
        .unsupported('SILENT WRONG: persistent y → y')
        .stateful()
        .gap('persistent.strip', 'persistent y', { expected: '' })
        .done(),
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
    feature('plus_eq', 'session')
        .unsupported('oak error on x+=1 and A(1)+=1')
        .stateful()
        .gap('pluseq.x', 'x=1; x+=1', { expected: '2' })
        .done(),
    feature('preincrement', 'session')
        .unsupported('SILENT WRONG: ++A → A; A++ oak error')
        .stateful()
        .gap('preinc.A', '++A', { expected: '...', notes: 'currently returns A' })
        .done(),
    feature('times_eq', 'session')
        .unsupported('oak error on A.*=3 / A./=2 / x^=2')
        .stateful()
        .gap('timeseq.elem', 'A=[1, 2]; A.*=3', { expected: '[3, 6]' })
        .done(),
];
