import assert from 'node:assert/strict';
import { Mathematica, mathematica } from '@sxo/mathematica';

const math = mathematica.d('x^3', 'x').toWolfram();
assert.ok(math.includes('x') || math.includes('3'), `math D got ${math}`);

const trig = mathematica.evaluate('Simplify[Sin[x]^2 + Cos[x]^2]');
assert.equal(trig.toWolfram(), '1');

const fact = mathematica.evaluate('5!');
assert.equal(fact.toWolfram(), '120');

const cmp = mathematica.evaluate('2 == 2');
assert.equal(cmp.toWolfram(), '1');

const integral = mathematica.evaluate('Integrate[x^2, x]');
assert.ok(integral.toWolfram().includes('x'), `Integrate got ${integral.toWolfram()}`);

const mapped = mathematica.evaluate('Map[Sin, {0, 1}]');
assert.ok(mapped.toWolfram().startsWith('{'), `Map got ${mapped.toWolfram()}`);

const mma = Mathematica.create({ autoSimplify: false });
assert.equal(mma.version(), mathematica.version());

console.log(`@sxo/mathematica ok: D=${math}, trig=${trig.toWolfram()}, fact=${fact.toWolfram()}`);
