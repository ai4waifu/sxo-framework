import assert from 'node:assert/strict';
import { d, version } from '@sxo/simple-math';

const v = version();
assert.equal(typeof v, 'string');
assert.match(v, /^\d+\.\d+\.\d+/);

assert.throws(
    () => d('x^2', 'x'),
    (err) => err instanceof Error && err.message.includes('off the current delivery route'),
);

console.log(`@sxo/simple-math ok: version=${v}`);
