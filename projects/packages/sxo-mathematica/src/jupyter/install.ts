/**
 * Jupyter kernelspec install / uninstall helpers for `@sxo/mathematica`.
 */

import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/** Default Jupyter kernel id. */
export const DEFAULT_KERNEL_NAME = 'sxo-mathematica';

/** Default Jupyter display name. */
export const DEFAULT_DISPLAY_NAME = 'SXO Mathematica';

/** Options for kernelspec install / uninstall. */
export type KernelspecOptions = {
    /** Kernel directory id (default `sxo-mathematica`). */
    name?: string;
    /** Optional prefix → `<prefix>/share/jupyter/kernels/<name>`. */
    prefix?: string;
    /** Absolute path to `bin/wolframscript.mjs` (defaults to this package). */
    wolframscriptPath?: string;
    /** Node executable for argv[0] (defaults to `process.execPath`). */
    nodePath?: string;
};

/** Shape written to `kernel.json`. */
export type KernelSpecJson = {
    argv: string[];
    display_name: string;
    language: string;
    interrupt_mode: string;
    metadata: Record<string, unknown>;
};

/**
 * Resolve the user-level Jupyter kernels directory for the current platform.
 */
export function userKernelsDir(): string {
    const home = homedir();
    switch (process.platform) {
        case 'win32': {
            const appdata = process.env.APPDATA;
            if (!appdata) throw new Error('APPDATA is not set');
            return path.join(appdata, 'jupyter', 'kernels');
        }
        case 'darwin':
            return path.join(home, 'Library', 'Jupyter', 'kernels');
        default:
            return path.join(home, '.local', 'share', 'jupyter', 'kernels');
    }
}

/**
 * Resolve the kernelspec directory for `name` under optional `--prefix`.
 */
export function resolveKernelDir(options: KernelspecOptions = {}): string {
    const name = options.name?.trim() || DEFAULT_KERNEL_NAME;
    if (options.prefix) {
        return path.join(path.resolve(options.prefix), 'share', 'jupyter', 'kernels', name);
    }
    return path.join(userKernelsDir(), name);
}

/**
 * Absolute path to this package's `bin/wolframscript.mjs`.
 */
export function defaultWolframscriptPath(): string {
    return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', 'bin', 'wolframscript.mjs');
}

/**
 * Build a kernelspec JSON object (does not write to disk).
 */
export function buildKernelSpec(options: KernelspecOptions = {}): KernelSpecJson {
    const nodePath = options.nodePath ?? process.execPath;
    const script = path.resolve(options.wolframscriptPath ?? defaultWolframscriptPath());
    return {
        argv: [nodePath, script, 'jupyter', 'kernel', '{connection_file}'],
        display_name: DEFAULT_DISPLAY_NAME,
        language: 'wolfram',
        interrupt_mode: 'message',
        metadata: {},
    };
}

/**
 * Install (or overwrite) a Jupyter kernelspec. Returns the kernel directory path.
 */
export function installKernelspec(options: KernelspecOptions = {}): string {
    const dir = resolveKernelDir(options);
    const spec = buildKernelSpec(options);
    mkdirSync(dir, { recursive: true });
    writeFileSync(path.join(dir, 'kernel.json'), `${JSON.stringify(spec, null, 2)}\n`, 'utf8');
    return dir;
}

/**
 * Remove an installed kernelspec directory. Returns whether it existed.
 */
export function uninstallKernelspec(options: KernelspecOptions = {}): boolean {
    const dir = resolveKernelDir(options);
    if (!existsSync(dir)) return false;
    rmSync(dir, { recursive: true, force: true });
    return true;
}
