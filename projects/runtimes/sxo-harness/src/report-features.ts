import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { fileURLToPath } from 'node:url';
import type { FeatureMatrix } from './matrix/types.js';
import { validateFeatureMatrix } from './matrix/validate.js';
import { toConsoleRows, toMarkdownTable } from './reporters/matrix.js';

type DialectId = 'mathematica' | 'matlab';

const HARNESS_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const RUNTIMES_ROOT = path.resolve(HARNESS_ROOT, '..');

/** Dialect-owned tests trees (not product `src/` / dist exports). */
const MATRIX_ENTRY: Record<DialectId, string> = {
    mathematica: path.join(RUNTIMES_ROOT, 'sxo-mathematica', 'tests', 'feature-matrix', 'index.ts'),
    matlab: path.join(RUNTIMES_ROOT, 'sxo-matlab', 'tests', 'feature-matrix', 'index.ts'),
};

function isDialectId(value: string | undefined): value is DialectId {
    return value === 'mathematica' || value === 'matlab';
}

export async function loadDialectFeatureMatrix(dialect: DialectId): Promise<FeatureMatrix> {
    const entry = MATRIX_ENTRY[dialect];
    const mod = (await import(pathToFileURL(entry).href)) as { featureMatrix: FeatureMatrix };
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

async function main(argv: string[] = process.argv): Promise<number> {
    const dialectArg = argv[2];
    const modeArg = argv[3] ?? 'markdown';
    if (!isDialectId(dialectArg)) {
        console.error('Usage: report-features <mathematica|matlab> [markdown|table]');
        console.error('Prefer: pnpm --filter @sxo/<dialect> report:features');
        return 1;
    }
    if (modeArg !== 'markdown' && modeArg !== 'table') {
        console.error('Mode must be markdown or table');
        return 1;
    }
    try {
        const out = await reportDialectFeatures(dialectArg, modeArg);
        if (out !== null) process.stdout.write(out);
        return 0;
    } catch (err) {
        console.error(err instanceof Error ? err.message : String(err));
        return 1;
    }
}

const isDirect = process.argv[1] !== undefined && /report-features\.[cm]?[jt]s$/.test(process.argv[1].replace(/\\/g, '/'));
if (isDirect) {
    process.exitCode = await main();
}
