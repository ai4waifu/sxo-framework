/**
 * Homepage lite probe against the **pinned** registry `@dxo/lite` (CPU fallback).
 * Does not require local wasm-pack / lite-unknown-wasm32.
 */
import assert from 'node:assert/strict';
import { createRuntime, version } from '@dxo/lite';

const rt = await createRuntime({ fallback: 'cpu' });
assert.equal(rt.capabilities.webglTensorBackend, false);
const a = rt.tensor([1, 2, 3, 4], [2, 2]);
const b = rt.tensor([1, 0, 0, 1], [2, 2]);
const out = await a.matmul(b).toArray();
assert.deepEqual(out, [1, 2, 3, 4]);
rt.destroy();
console.log(`lite-probe ok: version=${version()} backend=${rt.capabilities?.backend ?? 'n/a'}`);
