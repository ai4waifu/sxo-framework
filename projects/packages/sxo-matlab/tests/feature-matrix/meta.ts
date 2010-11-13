import { feature } from '@sxo/harness';

export const metaFeatures = [
    feature('which', 'meta')
        .unsupported('Command Form kept; Reject (no which runtime)')
        .pure()
        .gap('which.sin', 'which sin', {
            expected: '...',
            notes: 'parses as Command(which, sin); evaluate Rejects — not silent last-token',
        })
        .done(),
    feature('profile', 'meta')
        .unsupported('Command Form kept; Reject (no profile runtime)')
        .effectful()
        .gap('profile.on', 'profile on', {
            expected: '...',
            notes: 'parses as Command(profile, on); evaluate Rejects — not silent last-token',
        })
        .done(),
    feature('dbstop', 'meta')
        .unsupported('Command Form kept; Reject (no dbstop runtime)')
        .effectful()
        .gap('dbstop.if_error', 'dbstop if error', {
            expected: '...',
            notes: 'keyword bareword if is a Command arg; evaluate Rejects — not juxta/VM leak',
        })
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
        .unsupported('Command Form kept; Reject (no format runtime)')
        .effectful()
        .gap('format.long', 'format long', {
            expected: '...',
            notes: 'parses as Command(format, long); evaluate Rejects — not silent last-token',
        })
        .done(),
    feature('methods_meta', 'meta')
        .unsupported('Call Form residual / Reject (no methods runtime)')
        .pure()
        .gap('methods.double', "methods('double')", {
            expected: '...',
            notes: "paren call methods('double'), not command juxta; must not collapse to 'double'",
        })
        .done(),
    feature('builtin', 'meta').unsupported().pure().gap('builtin.sin', "builtin('sin', 0)", { expected: '0' }).done(),
];
