import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';

const require = createRequire(import.meta.url);
const platform = `${process.platform}-${process.arch}`;
const nativePath = resolve(`projects/platforms/native/sxo-${platform}/sxo.${platform === 'win32-x64' ? 'win32-x64-msvc' : platform}.node`);
const hosts = [
    { name: 'native', binding: require(nativePath), parse: (binding, source, dialect = 'maple') => binding.Expression.parse(source, dialect) },
];

if (process.argv[2]) {
    const wasmPath = resolve(process.argv[2]);
    hosts.push({
        name: 'wasm',
        binding: require(wasmPath),
        parse: (binding, source, dialect = 'maple') => new binding.Expression(source, dialect),
    });
}

for (const { name, binding, parse } of hosts) {
    const direct = binding.evaluate('1 / 3', 'maple');
    const parsed = parse(binding, '1 / 3');
    const parsedResult = parsed.evaluate();
    for (const expression of [direct, parsedResult]) {
        assert.equal(expression.dialect, 'maple', `${name}: explicit dialect`);
        assert.equal(expression.status, 'Exact', `${name}: exact status`);
        assert.equal(expression.coverage, 'Full', `${name}: full coverage`);
        assert.equal(expression.toString(), '1/3', `${name}: exact projection`);
    }

    const sequence = parse(binding, '1 / 3; 7');
    assert.equal(sequence.toString(), '(1 / 3); 7', `${name}: held source sequence`);
    const sequenceResult = sequence.evaluate();
    assert.equal(sequenceResult.status, 'Exact', `${name}: sequence status`);
    assert.equal(sequenceResult.toString(), '7', `${name}: sequence last result`);
    assert.equal(parsed.toString(), '(1 / 3)', `${name}: evaluation preserves held handle`);
    assert.throws(() => parse(binding, '1 +'), `${name}: invalid Maple syntax`);
    assert.throws(() => parse(binding, '1', 'unknown'), `${name}: unknown dialect`);

    const scalar = parse(binding, '7');
    assert.throws(() => scalar.toWolfram(), `${name}: no implicit Wolfram conversion`);
    assert.throws(() => scalar.toMatlab(), `${name}: no implicit MATLAB conversion`);
    assert.throws(() => scalar.d('x'), `${name}: unsupported derivative is explicit`);
    assert.throws(() => scalar.simplify(), `${name}: unsupported simplify is explicit`);
    assert.throws(() => scalar.evaluate(name === 'native' ? { strategy: 'simplify' } : 'simplify'), `${name}: no generic simplify bypass`);
    assert.throws(
        () => binding.evaluate('7', 'maple', name === 'native' ? { strategy: 'simplify' } : 'simplify'),
        `${name}: no top-level simplify bypass`,
    );
    assert.throws(() => binding.simplify('7', 'maple'), `${name}: simplify API stays unsupported`);
    assert.equal(scalar.isEqual(parse(binding, '7')), true, `${name}: Maple Form equality`);

    console.log(`${name}: Maple direct/parse/sequence/error/equality checks passed`);
}
