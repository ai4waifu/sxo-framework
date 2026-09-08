import { feature } from '@sxo/harness';

export const odeFeatures = [
    feature('ode45', 'ode')
        .unsupported('Call Form kept (all args); no ode45 solver runtime')
        .pure()
        .gap('ode45.strip', 'ode45(@(t,y)y, [0, 1], 1)', {
            expected: '...',
            notes: 'must keep three-arg call shape; not silent last-arg strip to 1',
        })
        .done(),
    feature('ode23', 'ode')
        .unsupported('Call Form kept (all args); no ode23 solver runtime')
        .pure()
        .gap('ode23.strip', 'ode23(@(t,y)y, [0, 1], 1)', {
            expected: '...',
            notes: 'same fidelity as ode45: keep call args, no last-arg strip',
        })
        .done(),
    feature('ode15s', 'ode')
        .unsupported('Call Form kept (all args); no ode15s solver runtime')
        .pure()
        .gap('ode15s.strip', 'ode15s(@(t,y)y, [0, 1], 1)', {
            expected: '...',
            notes: 'keep call args; not silent last-arg strip',
        })
        .done(),
    feature('ode113', 'ode')
        .unsupported('Call Form kept (all args); no ode113 solver runtime')
        .pure()
        .gap('ode113.strip', 'ode113(@(t,y)y, [0, 1], 1)', {
            expected: '...',
            notes: 'keep call args; not silent last-arg strip',
        })
        .done(),
    feature('dde23', 'ode')
        .unsupported('Call Form kept (all args); no dde23 solver runtime')
        .pure()
        .gap('dde23.strip', 'dde23(@(t,y,z)z, [1], 1, [0, 2])', {
            expected: '...',
            notes: 'keep full call; not silent last-arg strip to [0, 2]',
        })
        .done(),
    feature('odeset', 'ode')
        .unsupported('Nested Call Form kept; no ode45/odeset runtime')
        .pure()
        .gap('odeset.strip', "ode45(@(t,y)y, [0, 1], 1, odeset('RelTol', 1e-3))", {
            expected: '...',
            notes: 'outer ode45 must keep four args including odeset(...) — not collapse',
        })
        .done(),
    feature('odeset_opts', 'ode')
        .unsupported('odeset itself echoes; scientific 1e-6 becomes decimal')
        .pure()
        .gap('odeset.reltol', "odeset('RelTol', 1e-6)", { expected: '...' })
        .done(),
];
