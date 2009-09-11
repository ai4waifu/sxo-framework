import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import {
    buildKernelSpec,
    DEFAULT_DISPLAY_NAME,
    DEFAULT_KERNEL_NAME,
    installKernelspec,
    resolveKernelDir,
    uninstallKernelspec,
} from '@sxo/mathematica/jupyter';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bin = path.join(root, 'bin', 'wolframscript.mjs');

describe('jupyter kernelspec helpers', () => {
    it('builds kernel.json shape with wolframscript argv', () => {
        const spec = buildKernelSpec({
            nodePath: '/usr/bin/node',
            wolframscriptPath: '/opt/sxo/bin/wolframscript.mjs',
        });
        expect(spec.display_name).toBe(DEFAULT_DISPLAY_NAME);
        expect(spec.language).toBe('wolfram');
        expect(spec.argv).toEqual(['/usr/bin/node', path.resolve('/opt/sxo/bin/wolframscript.mjs'), 'jupyter', 'kernel', '{connection_file}']);
    });

    it('installs under --prefix and uninstalls', () => {
        const prefix = mkdtempSync(path.join(tmpdir(), 'sxo-jupyter-'));
        const name = 'sxo-test-kernel';
        try {
            const dir = installKernelspec({ prefix, name });
            expect(dir).toBe(resolveKernelDir({ prefix, name }));
            const kernelJson = path.join(dir, 'kernel.json');
            expect(existsSync(kernelJson)).toBe(true);
            const parsed = JSON.parse(readFileSync(kernelJson, 'utf8')) as {
                argv: string[];
                display_name: string;
                language: string;
            };
            expect(parsed.display_name).toBe(DEFAULT_DISPLAY_NAME);
            expect(parsed.language).toBe('wolfram');
            expect(parsed.argv).toContain('jupyter');
            expect(parsed.argv).toContain('kernel');
            expect(parsed.argv).toContain('{connection_file}');
            expect(parsed.argv.some((a) => a.includes('wolframscript'))).toBe(true);

            expect(uninstallKernelspec({ prefix, name })).toBe(true);
            expect(existsSync(dir)).toBe(false);
            expect(uninstallKernelspec({ prefix, name })).toBe(false);
        } finally {
            rmSync(prefix, { recursive: true, force: true });
        }
    });

    it('CLI jupyter install / uninstall with --prefix', () => {
        const prefix = mkdtempSync(path.join(tmpdir(), 'sxo-jupyter-cli-'));
        try {
            const install = spawnSync(process.execPath, [bin, 'jupyter', 'install', '--prefix', prefix, '--name', DEFAULT_KERNEL_NAME], {
                encoding: 'utf8',
                cwd: root,
            });
            expect(install.status, install.stderr).toBe(0);
            const kernelJson = path.join(prefix, 'share', 'jupyter', 'kernels', DEFAULT_KERNEL_NAME, 'kernel.json');
            expect(existsSync(kernelJson)).toBe(true);
            const parsed = JSON.parse(readFileSync(kernelJson, 'utf8')) as { argv: string[] };
            expect(parsed.argv.join(' ')).toMatch(/wolframscript/);
            expect(parsed.argv).toContain('{connection_file}');

            const uninstall = spawnSync(process.execPath, [bin, 'jupyter', 'uninstall', '--prefix', prefix, '--name', DEFAULT_KERNEL_NAME], {
                encoding: 'utf8',
                cwd: root,
            });
            expect(uninstall.status, uninstall.stderr).toBe(0);
            expect(existsSync(kernelJson)).toBe(false);
        } finally {
            rmSync(prefix, { recursive: true, force: true });
        }
    });
});
