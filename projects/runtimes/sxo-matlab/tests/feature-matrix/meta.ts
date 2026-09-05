import { feature } from '@sxo/harness';

export const metaFeatures = [
    feature('which', 'meta').unsupported('SILENT WRONG: which sin → sin').pure().gap('which.sin', 'which sin', { expected: '...' }).done(),
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
    feature('eval', 'meta').unsupported().effectful().gap('eval.plus', "eval('1+1')", { expected: '2' }).done(),
    feature('feval', 'meta').unsupported().pure().gap('feval.sin', "feval('sin', 0)", { expected: '0' }).done(),
    feature('func2str', 'meta')
        .unsupported('@ stripped: func2str(@sin) → func2str(sin)')
        .pure()
        .gap('func2str.sin', 'func2str(@sin)', { expected: "'sin'" })
        .done(),
    feature('exist', 'meta').unsupported().pure().gap('exist.sin', "exist('sin', 'builtin')", { expected: '5' }).done(),
    feature('format', 'meta')
        .unsupported('SILENT WRONG: format long → long')
        .effectful()
        .gap('format.long', 'format long', { expected: '...', notes: 'currently returns long' })
        .done(),
    feature('methods_meta', 'meta')
        .unsupported("SILENT WRONG: methods('double') → 'double'")
        .pure()
        .gap('methods.double', "methods('double')", { expected: '...', notes: "currently returns 'double'" })
        .done(),
    feature('builtin', 'meta').unsupported().pure().gap('builtin.sin', "builtin('sin', 0)", { expected: '0' }).done(),
];
