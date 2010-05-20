import { existsSync, statSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

const require = createRequire(import.meta.url);

/** Absolute path and fingerprint of the loaded N-API addon (acceptance identity). */
export type NativeBinaryIdentity = {
    path: string;
    packageName: string;
    triple: string;
    /** Binding `version()` string. */
    version: string;
    size: number;
    mtimeMs: number;
};

/** Options for host `evaluate` (strategy applied in one N-API crossing). */
export type EvaluateOptions = {
    /** `"none"` (default) or `"simplify"`. */
    strategy?: 'none' | 'simplify';
};

/** Opaque N-API expression handle (methods only; no arena identity). */
export type NativeExpression = {
    d(varName: string): NativeExpression;
    simplify(): NativeExpression;
    evaluate(options?: EvaluateOptions | null): NativeExpression;
    toString(): string;
    toWolfram(): string;
    toMatlab(): string;
    isEqual(other: NativeExpression): boolean;
    plotSvg(): string;
    dialect: string;
    /** Athena `ComputationStatus` name from the last evaluate (`Exact`, `Candidate`, …). */
    status: string;
    /** Coverage name (`Full`, `Partial`, `Unknown`, `Unsupported`). */
    coverage: string;
    /** Diagnostic summaries from the last evaluate. */
    diagnostics: string[];
};

/** Full native host ABI used by `@sxo/core` and dialect adapters. */
export type NativeBinding = {
    version(): string;
    expression(input: string, dialect?: string | null): NativeExpression;
    evaluate(input: string, dialect?: string | null, options?: EvaluateOptions | null): NativeExpression;
    d(input: string, varName: string, dialect?: string | null): NativeExpression;
    simplify(input: string, dialect?: string | null): NativeExpression;
    plotSvg(input: string, dialect?: string | null): string;
    /** Block until Jupyter kernel shutdown (connection file path). */
    runJupyterKernel(connectionFile: string): void;
};

function platformPackage(): { name: string; triple: string } {
    const { platform, arch } = process;
    if (platform === 'win32' && arch === 'x64') {
        return { name: '@sxo/sxo-win32-x64', triple: 'win32-x64-msvc' };
    }
    if (platform === 'darwin' && arch === 'arm64') {
        return { name: '@sxo/sxo-darwin-arm64', triple: 'darwin-arm64' };
    }
    if (platform === 'darwin' && arch === 'x64') {
        return { name: '@sxo/sxo-darwin-x64', triple: 'darwin-x64' };
    }
    if (platform === 'linux' && arch === 'x64') {
        return { name: '@sxo/sxo-linux-x64', triple: 'linux-x64-gnu' };
    }
    throw new Error(`unsupported platform ${platform}-${arch}`);
}

let cached: NativeBinding | null = null;
let cachedIdentity: NativeBinaryIdentity | null = null;

function resolveBinaryPath(): { name: string; triple: string; binary: string } {
    const { name, triple } = platformPackage();
    const pkgJson = require.resolve(`${name}/package.json`);
    const dir = path.dirname(pkgJson);
    const binary = path.join(dir, `sxo.${triple}.node`);
    if (!existsSync(binary)) {
        throw new Error(`native addon missing: ${binary} (run pnpm build:native)`);
    }
    return { name, triple, binary };
}

/** Load the platform N-API addon (cached). */
export function loadNative(): NativeBinding {
    if (cached) return cached;
    const { binary } = resolveBinaryPath();
    cached = require(binary) as NativeBinding;
    return cached;
}

/** Identity of the loaded native addon (path, size, mtime, version). */
export function nativeBinaryIdentity(): NativeBinaryIdentity {
    if (cachedIdentity) return cachedIdentity;
    const { name, triple, binary } = resolveBinaryPath();
    const binding = loadNative();
    const st = statSync(binary);
    cachedIdentity = {
        path: binary,
        packageName: name,
        triple,
        version: binding.version(),
        size: st.size,
        mtimeMs: st.mtimeMs,
    };
    return cachedIdentity;
}
