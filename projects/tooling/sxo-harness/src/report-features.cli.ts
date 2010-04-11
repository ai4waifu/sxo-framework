import { loadDialectFeatureMatrix, reportDialectFeatures } from './report-features.js';
import { listCasesByFlags } from './reporters/matrix.js';

type DialectId = 'mathematica' | 'matlab' | 'pari-gp';

function isDialectId(value: string | undefined): value is DialectId {
    return value === 'mathematica' || value === 'matlab' || value === 'pari-gp';
}

function parseFlagArgs(argv: string[]): { dialect?: string; mode: string; flags: string[] } {
    const positional: string[] = [];
    const flags: string[] = [];
    for (let i = 2; i < argv.length; i += 1) {
        const arg = argv[i]!;
        if (arg.startsWith('--flag=')) {
            const value = arg.slice('--flag='.length).trim();
            if (value)
                flags.push(
                    ...value
                        .split(',')
                        .map((f) => f.trim())
                        .filter(Boolean),
                );
            continue;
        }
        if (arg === '--flag' || arg === '-f') {
            const next = argv[i + 1];
            if (next && !next.startsWith('-')) {
                flags.push(
                    ...next
                        .split(',')
                        .map((f) => f.trim())
                        .filter(Boolean),
                );
                i += 1;
            }
            continue;
        }
        if (arg.startsWith('-')) continue;
        positional.push(arg);
    }
    const dialect = positional[0];
    let mode = positional[1] ?? 'markdown';
    if (mode === 'wrongs' || mode === 'wrong') {
        flags.push('wrong');
        mode = 'flags';
    }
    return { dialect, mode, flags };
}

async function main(argv: string[] = process.argv): Promise<number> {
    const { dialect: dialectArg, mode, flags } = parseFlagArgs(argv);
    if (!isDialectId(dialectArg)) {
        console.error('Usage: report-features <mathematica|matlab|pari-gp> [markdown|table|wrongs]');
        console.error('       report-features <mathematica|matlab|pari-gp> --flag=wrong[,upstream-athena]');
        console.error('Prefer: pnpm --filter @sxo/<dialect> report:features');
        return 1;
    }
    try {
        if (mode === 'flags' || flags.length > 0) {
            const matrix = await loadDialectFeatureMatrix(dialectArg);
            const want = flags.length > 0 ? flags : ['wrong'];
            const rows = listCasesByFlags(matrix, want);
            console.table(rows);
            console.error(`flagged cases: ${rows.length} (flags=${want.join(',')})`);
            return 0;
        }
        if (mode !== 'markdown' && mode !== 'table') {
            console.error('Mode must be markdown, table, or wrongs');
            return 1;
        }
        const out = await reportDialectFeatures(dialectArg, mode);
        if (out !== null) process.stdout.write(out);
        return 0;
    } catch (err) {
        console.error(err instanceof Error ? err.message : String(err));
        return 1;
    }
}

process.exitCode = await main();
