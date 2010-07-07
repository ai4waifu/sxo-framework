import { feature } from '@sxo/harness';

/** REPL / precision session controls — planned with typed PariGpSession. */
export const sessionFeatures = [
    feature('default_realprecision', 'session')
        .planned('default(realprecision, n) session option')
        .stateful()
        .gap('precision.set38', 'default(realprecision, 38)', {
            expected: '38',
            notes: 'needs PariGpSession + precision policy',
        })
        .done(),
    feature('assignment', 'session').planned('GP `=` binding in session').stateful().gap('assign.a', 'a = 3; a + 1', { expected: '4' }).done(),
];
