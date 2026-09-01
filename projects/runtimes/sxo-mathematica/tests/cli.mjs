import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFileSync, unlinkSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { adaptArgv } from '../dist/cli.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bin = path.join(root, 'bin', 'wolframscript.mjs');

function run(args) {
    return spawnSync(process.execPath, [bin, ...args], {
        encoding: 'utf8',
        cwd: root,
    });
}

{
    const adapted = adaptArgv(['node', 'wolframscript', '-code', '1+1']);
    assert.deepEqual(adapted.slice(2), ['code', '1+1']);
}

{
    const adapted = adaptArgv(['node', 'wolframscript', '-file', 'a.wl', '-print', 'all']);
    assert.deepEqual(adapted.slice(2), ['file', 'a.wl', '--print', 'all']);
}

{
    const adapted = adaptArgv(['node', 'wolframscript', 'script.wls', '5']);
    assert.deepEqual(adapted.slice(2), ['script', 'script.wls', '5']);
}

{
    const r = run(['version']);
    assert.equal(r.status, 0, r.stderr);
    assert.match(r.stdout, /wolframscript/);
}

{
    const r = run(['-code', 'Simplify[Sin[x]^2 + Cos[x]^2]']);
    assert.equal(r.status, 0, r.stderr);
    assert.equal(r.stdout.trim(), '1');
}

{
    const dir = mkdtempSync(path.join(tmpdir(), 'wolframscript-'));
    const file = path.join(dir, 't.wl');
    writeFileSync(file, '2+2\n');
    const r = run(['-file', file]);
    assert.equal(r.status, 0, r.stderr);
    assert.equal(r.stdout.trim(), '4');
    unlinkSync(file);
}

{
    const dir = mkdtempSync(path.join(tmpdir(), 'wolframscript-'));
    const file = path.join(dir, 't.wls');
    writeFileSync(file, '#!/usr/bin/env wolframscript\nPrint[2+2]\n');
    const r = run([file]);
    assert.equal(r.status, 0, r.stderr);
    assert.equal(r.stdout.trim(), '4');
    unlinkSync(file);
}

console.log('@sxo/mathematica wolframscript cli ok');
