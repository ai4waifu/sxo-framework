import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import type { FeatureMatrix } from './matrix/types.js';
import { validateFeatureMatrix } from './matrix/validate.js';
import { toConsoleRows, toMarkdownTable } from './reporters/matrix.js';

type DialectId = 'mathematica' | 'matlab';

const require = createRequire(import.meta.url);

/** Map CLI dialect id → npm package that owns the matrix. */
const DIALECT_PACKAGE: Record<DialectId, string> = {
    mathematica: '@sxo/mathematica',
    matlab: '@sxo/matlab',
};

type SxoPackageConfig = {
    featureMatrix?: string;
};

type PackageJson = {
    name?: string;
    sxo?: SxoPackageConfig;
};

/** Walk up from a resolved package entry until package.json with matching name. */
function resolvePackageRoot(pkgName: string): string {
    const entry = require.resolve(pkgName);
    let dir = path.dirname(entry);
    for (;;) {
        const candidate = path.join(dir, 'package.json');
        if (existsSync(candidate)) {
            const pkg = JSON.parse(readFileSync(candidate, 'utf8')) as PackageJson;
            if (pkg.name === pkgName) {
                return dir;
            }
        }
        const parent = path.dirname(dir);
        if (parent === dir) {
            throw new Error(`cannot locate package root for ${pkgName} (started at ${entry})`);
        }
        dir = parent;
    }
}

/**
 * Resolve the dialect-owned feature matrix entry from package.json `sxo.featureMatrix`.
 *
 * Harness does not hard-code package tree paths. Dialects declare the relative entry
 * under their own package root.
 */
export function resolveDialectFeatureMatrixEntry(dialect: DialectId): string {
    const pkgName = DIALECT_PACKAGE[dialect];
    const root = resolvePackageRoot(pkgName);
    const pkg = JSON.parse(readFileSync(path.join(root, 'package.json'), 'utf8')) as PackageJson;
    const rel = pkg.sxo?.featureMatrix?.trim();
    if (!rel) {
        throw new Error(`${pkgName} package.json missing sxo.featureMatrix (dialect-owned matrix entry)`);
    }
    return path.resolve(root, rel);
}

export async function loadDialectFeatureMatrix(dialect: DialectId): Promise<FeatureMatrix> {
    const entry = resolveDialectFeatureMatrixEntry(dialect);
    const mod = (await import(pathToFileURL(entry).href)) as { featureMatrix: FeatureMatrix };
    if (!mod.featureMatrix) {
        throw new Error(`${entry} must export featureMatrix`);
    }
    return mod.featureMatrix;
}

export async function reportDialectFeatures(dialect: DialectId, mode: 'markdown' | 'table' = 'markdown'): Promise<string | null> {
    const matrix = await loadDialectFeatureMatrix(dialect);
    const validation = validateFeatureMatrix(matrix);
    if (!validation.ok) {
        const detail = validation.issues.map((i) => `  - ${i.message}`).join('\n');
        throw new Error(`Invalid ${dialect} feature matrix:\n${detail}`);
    }
    if (mode === 'table') {
        console.table(toConsoleRows(matrix));
        return null;
    }
    return toMarkdownTable(matrix);
}
