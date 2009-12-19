import {
    ensureReady,
    type Expression as WasmExpression,
    d as wasmD,
    expression as wasmExpression,
    simplify as wasmSimplify,
    version as wasmVersion,
} from '@sxo/lite-unknown-wasm32';

export class Expression {
    readonly #inner: WasmExpression;

    constructor(inner: WasmExpression) {
        this.#inner = inner;
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

/** Initialize WASM (call once before other APIs in browser). */
export async function init(wasmUrl?: string): Promise<void> {
    await ensureReady(wasmUrl);
}

export function version(): string {
    return wasmVersion();
}

/** Simple-math only (Mathematica is `@sxo/mathematica` on Node). */
export function expression(input: string | Expression): Expression {
    if (input instanceof Expression) return input;
    return new Expression(wasmExpression(input, 'simple-math'));
}

export function d(input: string | Expression, varName: string): Expression {
    if (input instanceof Expression) return input.d(varName);
    return new Expression(wasmD(input, varName, 'simple-math'));
}

export function simplify(input: string | Expression): Expression {
    if (input instanceof Expression) return input.simplify();
    return new Expression(wasmSimplify(input, 'simple-math'));
}
