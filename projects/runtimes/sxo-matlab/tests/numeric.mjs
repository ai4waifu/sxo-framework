import assert from 'node:assert/strict';
import { matlab } from '@sxo/matlab';

const big = matlab.evaluate('99999999999999999999 + 1');
assert.equal(big.toMatlab(), '100000000000000000000');

const exact = matlab.evaluate('1/3 + 1/3 + 1/3');
assert.equal(exact.toMatlab(), '1');

console.log(`@sxo/matlab numeric ok: big=${big.toMatlab()}, exact=${exact.toMatlab()}`);
