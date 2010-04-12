/**
 * PARI/GP dialect surface for SXO.
 *
 * Placeholder API: evaluate/parse are not implemented yet.
 * Acceptance lives in `tests/feature-matrix/` and must stay present for every release.
 */
export class PariGpUnsupportedError extends Error {
    constructor(message = 'PARI/GP dialect surface is not implemented yet') {
        super(message);
        this.name = 'PariGpUnsupportedError';
    }
}

/** Explicit dialect tag used by hosts once wired. */
export const PARI_GP_DIALECT = 'pari-gp' as const;

/** Package identity for tooling / doctors. */
export function packageName(): '@sxo/pari-gp' {
    return '@sxo/pari-gp';
}

/** Placeholder evaluate entry — always throws until Form/lowering lands. */
export function evaluate(_input: string): never {
    throw new PariGpUnsupportedError();
}

/** Placeholder parse entry — always throws until oak + `GpForm` land. */
export function parse(_input: string): never {
    throw new PariGpUnsupportedError();
}
