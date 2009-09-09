import assert from 'node:assert/strict';
import { mathematica } from '@sxo/mathematica';

const big = mathematica.evaluate('99999999999999999999 + 1');
assert.equal(big.toWolfram(), '100000000000000000000');

const exact = mathematica.evaluate('1/3 + 1/3 + 1/3');
assert.equal(exact.toWolfram(), '1');

console.log(`@sxo/mathematica numeric ok: big=${big.toWolfram()}, exact=${exact.toWolfram()}`);
