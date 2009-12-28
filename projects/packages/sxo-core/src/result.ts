/** Outcome status for a CAS request (neutral contract, not dialect Form). */
export type ResultStatus = 'ok' | 'partial' | 'error' | 'unsupported';

/** Neutral result envelope used by core/lite adapters. */
export type CasResult<T = unknown> = {
    status: ResultStatus;
    value?: T;
    diagnostics?: import('./diagnostics.js').Diagnostic[];
};
