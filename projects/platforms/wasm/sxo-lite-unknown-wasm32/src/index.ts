/// <reference path="./wasm-bindgen.d.ts" />

import init, {
    Expression as WasmExpression,
    d as wasmD,
    expression as wasmExpression,
    simplify as wasmSimplify,
    version as wasmVersion,
} from '../lib/sxo_lite.js';

let ready: Promise<void> | null = null;

export async function ensureReady(wasmUrl?: string): Promise<void> {
    if (!ready) {
        ready = init(wasmUrl ? { module_or_path: wasmUrl } : undefined).then(() => undefined);
    }
    await ready;
}

export { WasmExpression as Expression, wasmD as d, wasmExpression as expression, wasmSimplify as simplify, wasmVersion as version };
