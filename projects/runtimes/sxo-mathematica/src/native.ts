import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

const require = createRequire(import.meta.url);

export type NativeExpression = {
    d(varName: string): NativeExpression;
    simplify(): NativeExpression;
    evaluate(): NativeExpression;
    toString(): string;
    toWolfram(): string;
    toMatlab(): string;
    isEqual(other: NativeExpression): boolean;
    dialect: string;
};

export type NativeBinding = {
    version(): string;
    expression(input: string, dialect?: string | null): NativeExpression;
    d(input: string, varName: string, dialect?: string | null): NativeExpression;
    simplify(input: string, dialect?: string | null): NativeExpression;
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

export function loadNative(): NativeBinding {
    if (cached) return cached;
    const { name, triple } = platformPackage();
    const pkgJson = require.resolve(`${name}/package.json`);
    const dir = path.dirname(pkgJson);
    const binary = path.join(dir, `sxo.${triple}.node`);
    if (!existsSync(binary)) {
        throw new Error(`native addon missing: ${binary} (run pnpm build:native)`);
    }
    cached = require(binary) as NativeBinding;
    return cached;
}

export const MATH_DIALECT = 'mathematica';
