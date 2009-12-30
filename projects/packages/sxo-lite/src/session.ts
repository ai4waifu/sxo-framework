import {
    ensureReady,
    type Expression as WasmExpression,
    d as wasmD,
    expression as wasmExpression,
    simplify as wasmSimplify,
    version as wasmVersion,
} from '@sxo/lite-unknown-wasm32';

import { Expression } from './expression.js';

export type WasmBinding = {
    version(): string;
    expression(input: string, dialect?: string | null): WasmExpression;
    d(input: string, varName: string, dialect?: string | null): WasmExpression;
    simplify(input: string, dialect?: string | null): WasmExpression;
};

/**
 * WASM-backed Session facade.
 *
 * Call {@link init} (or construct after `ensureReady`) before use in the browser.
 */
export class Session {
    readonly #binding: WasmBinding;

    constructor(binding: WasmBinding = defaultBinding()) {
        this.#binding = binding;
    }

    version(): string {
        return this.#binding.version();
    }

    /**
     * @internal Host binding for dialect adapters that still call WASM ABI directly.
     */
    get binding(): WasmBinding {
        return this.#binding;
    }

    expressionFromWasm(inner: WasmExpression): Expression {
        return Expression.fromWasm(inner);
    }
}

function defaultBinding(): WasmBinding {
    return {
        version: () => wasmVersion(),
        expression: (input, dialect) => wasmExpression(input, dialect ?? undefined),
        d: (input, varName, dialect) => wasmD(input, varName, dialect ?? undefined),
        simplify: (input, dialect) => wasmSimplify(input, dialect ?? undefined),
    };
}

/** Initialize WASM (call once before other APIs in browser). */
export async function init(wasmUrl?: string): Promise<void> {
    await ensureReady(wasmUrl);
}
