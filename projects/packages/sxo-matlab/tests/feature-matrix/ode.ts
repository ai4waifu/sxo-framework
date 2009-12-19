import { feature } from '@sxo/harness';

export const odeFeatures = [
    feature('ode45', 'ode')
        .unsupported('SILENT WRONG: ode45(@(t,y)y,[0,1],1) → 1 (strips to last arg)')
        .pure()
        .gap('ode45.strip', 'ode45(@(t,y)y, [0, 1], 1)', { expected: '...', notes: 'currently returns 1' })
        .done(),
    feature('ode23', 'ode')
        .unsupported('SILENT WRONG: ode23(@(t,y)y,[0,1],1) → 1 (same strip pattern as ode45)')
        .pure()
        .gap('ode23.strip', 'ode23(@(t,y)y, [0, 1], 1)', { expected: '...', notes: 'currently returns 1' })
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
    feature('odeset', 'ode')
        .unsupported('SILENT WRONG: ode45(..., odeset(...)) collapses to odeset(...) last arg')
        .pure()
        .gap('odeset.strip', "ode45(@(t,y)y, [0, 1], 1, odeset('RelTol', 1e-3))", {
            expected: '...',
            notes: "currently returns odeset('RelTol', 0.001)",
        })
        .done(),
    feature('odeset_opts', 'ode')
        .unsupported('odeset itself echoes; scientific 1e-6 becomes decimal')
        .pure()
        .gap('odeset.reltol', "odeset('RelTol', 1e-6)", { expected: '...' })
        .done(),
];
