/** WASM host capabilities for `@sxo/lite` (no native / Jupyter face). */
export type HostRuntime = 'wasm';

/** Capability contract shared with `@sxo/core` shape, values differ by host. */
export type Capabilities = {
    readonly host: HostRuntime;
    /** Jupyter is native-only. */
    readonly jupyter: false;
    /** Always true on `@sxo/lite`. */
    readonly wasm: true;
};

/** Fixed capability set for the WASM product face. */
export const CAPABILITIES: Capabilities = {
    host: 'wasm',
    jupyter: false,
    wasm: true,
};
