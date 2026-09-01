import { mkdtempSync, unlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { describe, expect, it } from 'vitest';
import { adaptArgv } from '@sxo/mathematica/cli';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bin = path.join(root, 'bin', 'wolframscript.mjs');

function run(args: string[]) {
    return spawnSync(process.execPath, [bin, ...args], {
        encoding: 'utf8',
        cwd: root,
    });
}

describe('wolframscript cli', () => {
    it('adapts -code', () => {
        expect(adaptArgv(['node', 'wolframscript', '-code', '1+1']).slice(2)).toEqual(['code', '1+1']);
    });

    it('adapts -file with -print all', () => {
        expect(adaptArgv(['node', 'wolframscript', '-file', 'a.wl', '-print', 'all']).slice(2)).toEqual(['file', 'a.wl', '--print', 'all']);
    });

    it('adapts script path', () => {
        expect(adaptArgv(['node', 'wolframscript', 'script.wls', '5']).slice(2)).toEqual(['script', 'script.wls', '5']);
    });

    it('prints version', () => {
        const r = run(['version']);
        expect(r.status, r.stderr).toBe(0);
        expect(r.stdout).toMatch(/wolframscript/);
    });

    it('evaluates -code', () => {
        const r = run(['-code', 'Simplify[Sin[x]^2 + Cos[x]^2]']);
        expect(r.status, r.stderr).toBe(0);
        expect(r.stdout.trim()).toBe('1');
    });

    it('evaluates -file', () => {
        const dir = mkdtempSync(path.join(tmpdir(), 'wolframscript-'));
        const file = path.join(dir, 't.wl');
        writeFileSync(file, '2+2\n');
        const r = run(['-file', file]);
        expect(r.status, r.stderr).toBe(0);
        expect(r.stdout.trim()).toBe('4');
        unlinkSync(file);
    });

    it('runs shebang script', () => {
        const dir = mkdtempSync(path.join(tmpdir(), 'wolframscript-'));
        const file = path.join(dir, 't.wls');
        writeFileSync(file, '#!/usr/bin/env wolframscript\nPrint[2+2]\n');
        const r = run([file]);
        expect(r.status, r.stderr).toBe(0);
        expect(r.stdout.trim()).toBe('4');
        unlinkSync(file);
    });
});
