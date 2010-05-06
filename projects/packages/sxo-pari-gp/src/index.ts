/**
 * PARI/GP dialect surface for SXO.
 *
 * Scaffold only: parse / evaluate throw until `oak-pari` + `GpForm` + Athena
 * lowering land. Acceptance lives in `tests/feature-matrix/` and must stay
 * present for every release.
 */

/** Thrown by placeholder API until the dialect is wired. */
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

/**
 * PARI/GP **frontend** placeholder — public entry of `@sxo/pari-gp`.
 *
 * Mirrors other dialect packages' class shape so hosts can depend on the API
 * surface before oak / Form / N-API wiring exists.
 */
export class PariGp {
    private constructor() {}

    /** Create a PARI/GP frontend instance. */
    static create(): PariGp {
        return new PariGp();
    }

    /** Placeholder parse — always throws. */
    parse(_input: string): never {
        throw new PariGpUnsupportedError('pari-gp: parse not implemented (needs oak-pari → GpForm)');
    }

    /** Placeholder evaluate — always throws. */
    evaluate(_input: string): never {
        throw new PariGpUnsupportedError('pari-gp: evaluate not implemented (needs GpForm lowering)');
    }
}

/** Shared default frontend (stateless scaffold). */
export const pariGp = PariGp.create();

/** @deprecated Prefer {@link PariGp.create} / {@link pariGp}. */
export function parse(input: string): never {
    return pariGp.parse(input);
}

/** @deprecated Prefer {@link PariGp.create} / {@link pariGp}. */
export function evaluate(input: string): never {
    return pariGp.evaluate(input);
}
