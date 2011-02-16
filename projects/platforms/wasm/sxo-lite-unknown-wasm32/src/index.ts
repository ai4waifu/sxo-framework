/// <reference path="./wasm-bindgen.d.ts" />

import init, {
    Expression as WasmExpression,
    d as wasmD,
    evaluate as wasmEvaluate,
    expression as wasmExpression,
    simplify as wasmSimplify,
    version as wasmVersion,
} from '../lib/sxo_lite.js';

let ready: Promise<void> | null = null;

async function loadWasm(moduleOrPath?: string): Promise<void> {
    if (moduleOrPath) {
        await init({ module_or_path: moduleOrPath });
        return;
    }
    try {
        await init();
    } catch {
        // wasm-pack `--target web` defaults to `fetch(import.meta.url)`. Node/Vitest has no fetch for file URLs.
        const { readFileSync } = await import('node:fs');
        const { dirname, join } = await import('node:path');
        const { fileURLToPath } = await import('node:url');
        const here = dirname(fileURLToPath(import.meta.url));
        const wasmBytes = readFileSync(join(here, '../lib/sxo_lite_bg.wasm'));
        await init(wasmBytes);
    }
}

export async function ensureReady(wasmUrl?: string): Promise<void> {
    if (!ready) {
        ready = loadWasm(wasmUrl);
    }
    await ready;
}

export {
    WasmExpression as Expression,
    wasmD as d,
    wasmEvaluate as evaluate,
    wasmExpression as expression,
    wasmSimplify as simplify,
    wasmVersion as version,
};
