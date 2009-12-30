import type { Expression as WasmExpression } from '@sxo/lite-unknown-wasm32';

/**
 * Opaque expression value owned by a {@link Session}.
 *
 * String syntax parsing belongs to explicit dialect packages, not `@sxo/lite`.
 */
export class Expression {
    readonly #inner: WasmExpression;

    /** @internal */
    constructor(inner: WasmExpression) {
        this.#inner = inner;
    }

    /** @internal */
    static fromWasm(inner: WasmExpression): Expression {
        return new Expression(inner);
    }

    /** @internal */
    get wasm(): WasmExpression {
        return this.#inner;
    }

    d(varName: string): Expression {
        return new Expression(this.#inner.d(varName));
    }

    simplify(): Expression {
        return new Expression(this.#inner.simplify());
    }

    toString(): string {
        return this.#inner.toString();
    }

    isEqual(other: Expression): boolean {
        return this.#inner.isEqual(other.#inner);
    }
}
