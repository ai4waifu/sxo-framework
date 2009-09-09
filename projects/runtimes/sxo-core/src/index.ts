import { CORE_DIALECT, loadNative, type NativeExpression } from './native.js';

export class Expression {
    readonly #inner: NativeExpression;

    constructor(inner: NativeExpression) {
        this.#inner = inner;
    }

    static fromNative(inner: NativeExpression): Expression {
        return new Expression(inner);
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

    toDSL(): string {
        return this.#inner.toDSL();
    }

    isEqual(other: Expression): boolean {
        return this.#inner.isEqual(other.#inner);
    }
}

/** Engine version (native). */
export function version(): string {
    return loadNative().version();
}

/** Parse a simple-math expression (Mathematica is `@sxo/mathematica` only). */
export function expression(input: string | Expression): Expression {
    if (input instanceof Expression) return input;
    return Expression.fromNative(loadNative().expression(input, CORE_DIALECT));
}

export function d(input: string | Expression, varName: string): Expression {
    if (input instanceof Expression) return input.d(varName);
    return Expression.fromNative(loadNative().d(input, varName, CORE_DIALECT));
}

export function simplify(input: string | Expression): Expression {
    if (input instanceof Expression) return input.simplify();
    return Expression.fromNative(loadNative().simplify(input, CORE_DIALECT));
}
