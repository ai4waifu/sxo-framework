import type { NativeExpression } from './native.js';

/**
 * Opaque expression value owned by a {@link Session}.
 *
 * String syntax parsing belongs to explicit dialect packages, not `@sxo/core`.
 */
export class Expression {
    readonly #inner: NativeExpression;

    /** @internal */
    constructor(inner: NativeExpression) {
        this.#inner = inner;
    }

    /** @internal */
    static fromNative(inner: NativeExpression): Expression {
        return new Expression(inner);
    }

    /** @internal */
    get native(): NativeExpression {
        return this.#inner;
    }

    d(varName: string): Expression {
        return new Expression(this.#inner.d(varName));
    }

    simplify(): Expression {
        return new Expression(this.#inner.simplify());
    }

    /** Evaluate via dialect `lower_request` (preserves Athena status / coverage). */
    evaluate(): Expression {
        return new Expression(this.#inner.evaluate());
    }

    toString(): string {
        return this.#inner.toString();
    }

    isEqual(other: Expression): boolean {
        return this.#inner.isEqual(other.#inner);
    }

    /** Athena computation status name from the last evaluate. */
    get status(): string {
        return this.#inner.status;
    }

    /** Coverage name from the last evaluate. */
    get coverage(): string {
        return this.#inner.coverage;
    }

    /** Diagnostic summaries from the last evaluate. */
    get diagnostics(): string[] {
        return this.#inner.diagnostics;
    }
}
