// Browser chunks on this toolchain do not resolve bare npm specifiers, so the
// lite runtime is vendored verbatim as a static asset (public/vendor/dxo-lite/)
// with its wasm32 dynamic import rewritten to a sibling file. Types still come
// from the published package and are erased at compile time. The specifier is
// kept in a variable so the static assembler leaves the URL untouched.
import type { LiteRuntime } from '@dxo/lite';

export type LiteProbe = {
    version: string;
    backend: string;
    webgpu: boolean;
    /** DXO GPU WASM facade readiness (0.0.9 published field was titanWgpuReady). */
    gpuBackendReady: boolean;
    webglTensorBackend: false;
    tensorClass: string;
    matmulOk: boolean;
    wasmVersion: string;
    wasmInterim: boolean;
};

type LiteModule = {
    createRuntime: (options?: {
        fallback?: 'cpu';
        wasm?: { url: URL };
        disable?: boolean;
    }) => Promise<LiteRuntime>;
    version: () => string;
};

let liteModulePromise: Promise<LiteModule> | undefined;

/** Load the vendored lite runtime module; the browser resolves the static URL. */
function loadLiteModule(): Promise<LiteModule> {
    if (!liteModulePromise) {
        // This file is mirrored to <root>/lib/lite-probe.js and public/** lands
        // at the deploy root, so the vendor entry is a sibling of lib/ both at
        // assemble time and in the browser.
        const url = new URL('../vendor/dxo-lite/index.mjs', import.meta.url).href;
        liteModulePromise = import(url) as Promise<LiteModule>;
    }
    return liteModulePromise;
}

/** Options for pinned registry lite runtime (interim WASM via lite-unknown-wasm32). */
export function homepageLiteOptions() {
    return {
        fallback: 'cpu' as const,
        wasm: typeof location !== 'undefined' ? { url: new URL('/dxo_lite_bg.wasm', location.origin) } : undefined,
    };
}

function readGpuBackendReady(capabilities: { gpuBackendReady?: boolean; titanWgpuReady?: boolean }): boolean {
    return capabilities.gpuBackendReady ?? capabilities.titanWgpuReady ?? false;
}

/** Runtime probe for the lite facade (async createRuntime + CPU fallback + interim WASM). */
export async function probeLite(): Promise<LiteProbe> {
    const lite = await loadLiteModule();
    const rt = await lite.createRuntime(homepageLiteOptions());
    try {
        const a = rt.tensor([1, 2, 3, 4], [2, 2]);
        const b = rt.tensor([1, 0, 0, 1], [2, 2]);
        const c = a.matmul(b);
        const out = await c.toArray();
        return {
            version: lite.version(),
            backend: rt.capabilities.backend,
            webgpu: rt.capabilities.webgpu,
            gpuBackendReady: readGpuBackendReady(rt.capabilities),
            webglTensorBackend: rt.capabilities.webglTensorBackend,
            tensorClass: a.constructor.name,
            matmulOk: out[0] === 1 && out[1] === 2 && out[2] === 3 && out[3] === 4,
            wasmVersion: rt.capabilities.wasm?.version ?? '(not in this release)',
            wasmInterim: rt.capabilities.wasm?.interimHostF32 ?? false,
        };
    } finally {
        rt.destroy();
    }
}

/** Open a lite runtime for the playground; returns the runtime and its version. */
export async function openHomepageRuntime(): Promise<{ runtime: LiteRuntime; version: string }> {
    const lite = await loadLiteModule();
    const runtime = await lite.createRuntime(homepageLiteOptions());
    return { runtime, version: lite.version() };
}
