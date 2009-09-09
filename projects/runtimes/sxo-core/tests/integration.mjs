import assert from 'node:assert/strict';
import { d, version } from '@sxo/core';

const v = version();
assert.equal(typeof v, 'string');
assert.match(v, /^\d+\.\d+\.\d+/);

assert.throws(
    () => d('x^3', 'x'),
    (err) => err instanceof Error && err.message.includes('off the current delivery route'),
);

console.log(`@sxo/core ok: version=${v}`);
