/** Native host capabilities for `@sxo/core` (no WASM face). */
export type HostRuntime = 'native';

/** Capability contract shared with `@sxo/lite` shape, values differ by host. */
export type Capabilities = {
    readonly host: HostRuntime;
    /** Jupyter / ZMQ kernel is available on the native host. */
    readonly jupyter: boolean;
    /** Always false on `@sxo/core`. */
    readonly wasm: false;
};

/** Fixed capability set for the native product face. */
export const CAPABILITIES: Capabilities = {
    host: 'native',
    jupyter: true,
    wasm: false,
};
