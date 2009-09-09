import assert from 'node:assert/strict';
import { Matlab, matlab } from '@sxo/matlab';

const ml = matlab.d('x^3', 'x').toMatlab();
assert.ok(ml.includes('x') || ml.includes('3'), `matlab diff got ${ml}`);

const mlTrig = matlab.evaluate('sin(x)^2 + cos(x)^2');
assert.equal(mlTrig.toMatlab(), '1');

const mlMulti = matlab.evaluate('1; 2+2');
assert.equal(mlMulti.toMatlab(), '4');

const matrix = matlab.parse('[1, 2; 3, 4]');
assert.equal(matrix.toMatlab(), '[1, 2; 3, 4]');

const integral = matlab.evaluate('int(x^2, x)');
assert.ok(integral.toMatlab().includes('x'), `int got ${integral.toMatlab()}`);

const cmp = matlab.evaluate('3 > 2');
assert.equal(cmp.toMatlab(), '1');

const mlInst = Matlab.create({ autoSimplify: false });
assert.equal(mlInst.version(), matlab.version());

console.log(`@sxo/matlab ok: diff=${ml}, trig=${mlTrig.toMatlab()}, matrix=${matrix.toMatlab()}`);
