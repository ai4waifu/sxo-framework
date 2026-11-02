/**
 * TypeScript rebind over wasm-bindgen glue in `../lib/`.
 * Public entry is compiled to `../dist/` — do not import `lib/` from apps.
 */
import init, { addF32, isInterimHostF32, matmulF32, version } from './dxo_lite_glue.js';
export { addF32, isInterimHostF32, matmulF32, version };
export default init;
/** URL of the packaged `.wasm` (stable relative to this module after tsc → dist/). */
export function wasmAssetUrl() {
    return new URL('../lib/dxo_lite_bg.wasm', import.meta.url);
}
//# sourceMappingURL=index.js.map