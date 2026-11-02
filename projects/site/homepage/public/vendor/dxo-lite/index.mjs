/**
 * @dxo/lite — browser/Worker runtime facade.
 *
 * TS owns async init, capabilities, explicit CPU fallback, and synchronous Tensor composition.
 * Interim path loads `./lite-wasm32.mjs` f32 kernels (GPU WASM facade not linked yet).
 * This layer never retains `GPUAdapter` / `GPUDevice` handles.
 * WebGL is never a tensor backend.
 */
/** Set true when WASM GPU facade is wired; until then compute stays host f32 (± interim WASM). */
export const GPU_BACKEND_READY = false;
function getGpu() {
    const nav = globalThis;
    return nav.navigator?.gpu;
}
function numel(shape) {
    return shape.reduce((n, d) => n * d, 1);
}
function assertShape(dataLen, shape) {
    const n = numel(shape);
    if (dataLen !== n) {
        throw new Error(`data length ${dataLen} does not match shape product ${n} (shape=[${shape.join(',')}])`);
    }
}
function matmulJs(a, aShape, b, bShape) {
    if (aShape.length !== 2 || bShape.length !== 2) {
        throw new Error('lite matmul requires rank-2 tensors');
    }
    const [m, k] = aShape;
    const [k2, n] = bShape;
    if (k !== k2)
        throw new Error(`matmul inner dim mismatch: ${k} vs ${k2}`);
    const out = new Float32Array(m * n);
    for (let i = 0; i < m; i++) {
        for (let j = 0; j < n; j++) {
            let sum = 0;
            for (let t = 0; t < k; t++)
                sum += a[i * k + t] * b[t * n + j];
            out[i * n + j] = sum;
        }
    }
    return { data: out, shape: [m, n] };
}
function addJs(a, b) {
    if (a.length !== b.length)
        throw new Error(`add length mismatch: ${a.length} vs ${b.length}`);
    const out = new Float32Array(a.length);
    for (let i = 0; i < a.length; i++)
        out[i] = a[i] + b[i];
    return out;
}
function matmulWith(kernels, a, aShape, b, bShape) {
    if (aShape.length !== 2 || bShape.length !== 2) {
        throw new Error('lite matmul requires rank-2 tensors');
    }
    const [m, k] = aShape;
    const [k2, n] = bShape;
    if (k !== k2)
        throw new Error(`matmul inner dim mismatch: ${k} vs ${k2}`);
    if (kernels) {
        return { data: kernels.matmulF32(a, m, k, b, n), shape: [m, n] };
    }
    return matmulJs(a, aShape, b, bShape);
}
function addWith(kernels, a, b) {
    if (kernels)
        return kernels.addF32(a, b);
    return addJs(a, b);
}
async function loadWasmKernels(options) {
    if (options?.disable)
        return null;
    try {
        const mod = await import('./lite-wasm32.mjs');
        const init = mod.default;
        if (options?.url) {
            await init({ module_or_path: options.url });
        }
        else if (typeof process !== 'undefined' && process.versions?.node) {
            const { readFile } = await import('node:fs/promises');
            const { createRequire } = await import('node:module');
            const require = createRequire(import.meta.url);
            // Prefer package export of lib/dxo_lite_bg.wasm (not beside dist/).
            const wasmPath = require.resolve('./lite-wasm32.mjs/dxo_lite_bg.wasm');
            await init({ module_or_path: await readFile(wasmPath) });
        }
        else {
            await init();
        }
        return {
            matmulF32: mod.matmulF32,
            addF32: mod.addF32,
            version: mod.version,
            isInterimHostF32: mod.isInterimHostF32,
        };
    }
    catch {
        return null;
    }
}
/** Dense float32 tensor. Ops synchronously return composable handles; observation is async. */
export class Tensor {
    #data;
    #kernels;
    shape;
    device;
    /** @internal */
    constructor(data, shape, device, kernels) {
        this.#data = data;
        this.#kernels = kernels;
        this.shape = Object.freeze([...shape]);
        this.device = device;
    }
    numel() {
        return this.#data.length;
    }
    matmul(other) {
        const { data, shape } = matmulWith(this.#kernels, this.#data, this.shape, other.#data, other.shape);
        return new Tensor(data, shape, this.device, this.#kernels);
    }
    add(other) {
        if (this.shape.length !== other.shape.length || this.shape.some((d, i) => d !== other.shape[i])) {
            throw new Error(`add requires identical shapes: [${this.shape}] vs [${other.shape}]`);
        }
        return new Tensor(addWith(this.#kernels, this.#data, other.#data), this.shape, this.device, this.#kernels);
    }
    /** Explicit host readback (copy). */
    async toCpu() {
        return Array.from(this.#data);
    }
    /** Alias for {@link toCpu}. */
    async toArray() {
        return this.toCpu();
    }
    /** Wait until pending device work for this handle is complete. CPU resolves immediately. */
    async ready() { }
    /** Scalar read when `numel === 1`. */
    async item() {
        if (this.numel() !== 1)
            throw new Error(`item() requires a scalar tensor, got shape [${this.shape.join(',')}]`);
        return this.#data[0];
    }
}
class RuntimeImpl {
    capabilities;
    #kernels;
    #destroyed = false;
    constructor(capabilities, kernels) {
        this.capabilities = capabilities;
        this.#kernels = kernels;
    }
    tensor(data, shape) {
        this.#assertLive();
        assertShape(data.length, shape);
        return new Tensor(Float32Array.from(data), shape, this.capabilities.backend, this.#kernels);
    }
    zeros(shape) {
        this.#assertLive();
        return new Tensor(new Float32Array(numel(shape)), shape, this.capabilities.backend, this.#kernels);
    }
    ones(shape) {
        this.#assertLive();
        const n = numel(shape);
        const data = new Float32Array(n);
        data.fill(1);
        return new Tensor(data, shape, this.capabilities.backend, this.#kernels);
    }
    destroy() {
        this.#destroyed = true;
    }
    #assertLive() {
        if (this.#destroyed)
            throw new Error('LiteRuntime has been destroyed');
    }
}
function cpuCapabilities(wasm) {
    return {
        backend: 'cpu',
        webgpu: false,
        gpuBackendReady: GPU_BACKEND_READY,
        webglTensorBackend: false,
        wasm,
        features: [],
        limits: {},
        dtype: { f32: true, f16: false },
    };
}
/**
 * Probe WebGPU adapter metadata without acquiring or retaining a device.
 * The adapter handle is dropped before returning.
 */
async function probeWebGpuCapabilities(options) {
    const gpu = getGpu();
    if (!gpu)
        return null;
    const adapter = await gpu.requestAdapter({
        powerPreference: options.powerPreference ?? 'high-performance',
    });
    if (!adapter)
        return null;
    const required = options.requiredFeatures ?? [];
    for (const feat of required) {
        if (!adapter.features.has(feat)) {
            throw new Error(`WebGPU adapter missing required feature '${feat}'`);
        }
    }
    const features = [...adapter.features.values()];
    let adapterInfo;
    try {
        const info = adapter.info ?? (await adapter.requestAdapterInfo?.());
        if (info) {
            adapterInfo = {
                vendor: info.vendor,
                architecture: info.architecture,
                device: info.device,
                description: info.description,
            };
        }
    }
    catch {
        /* optional */
    }
    const limits = {};
    for (const [k, v] of Object.entries(adapter.limits ?? {})) {
        if (typeof v === 'number')
            limits[k] = v;
    }
    return {
        backend: GPU_BACKEND_READY ? 'webgpu' : 'cpu',
        webgpu: true,
        gpuBackendReady: GPU_BACKEND_READY,
        webglTensorBackend: false,
        features,
        limits,
        dtype: { f32: true, f16: adapter.features.has('shader-f16') },
        adapterInfo,
    };
}
function gpuBackendNotReadyError() {
    return new Error("DXO GPU WASM facade is not ready; pass { fallback: 'cpu' } for host tensors until WASM is linked. WebGL is not a DXO tensor backend.");
}
function webGpuUnavailableError() {
    return new Error("WebGPU unavailable (no navigator.gpu). Pass { fallback: 'cpu' } for host tensors, or use a WebGPU-capable browser. WebGL is not a DXO tensor backend.");
}
function withWasmCap(base, kernels) {
    if (!kernels)
        return base;
    return {
        ...base,
        wasm: {
            version: kernels.version(),
            interimHostF32: kernels.isInterimHostF32(),
        },
    };
}
/**
 * Explicit async runtime bootstrap.
 * Never silently falls back to WebGL; never retains wgpu adapter/device in TS.
 */
export async function createRuntime(options = {}) {
    const fallback = options.fallback ?? 'error';
    const kernels = await loadWasmKernels(options.wasm);
    if (GPU_BACKEND_READY) {
        // Future: dynamic import WASM facade → dxo-core → GPU session.
        throw new Error('DXO GPU WASM facade hook is not implemented yet');
    }
    try {
        const probed = await probeWebGpuCapabilities(options);
        if (probed) {
            if (fallback === 'error') {
                throw gpuBackendNotReadyError();
            }
            return new RuntimeImpl(withWasmCap(probed, kernels), kernels);
        }
    }
    catch (err) {
        if (fallback === 'error')
            throw err;
    }
    if (fallback === 'cpu') {
        return new RuntimeImpl(withWasmCap(cpuCapabilities(), kernels), kernels);
    }
    if (!getGpu()) {
        throw webGpuUnavailableError();
    }
    throw new Error("WebGPU adapter request failed. Pass { fallback: 'cpu' } for host tensors. WebGL is not a DXO tensor backend.");
}
/** Package identity string (not npm semver). */
export function version() {
    return '0.1.0-dxo-lite';
}
//# sourceMappingURL=index.js.map