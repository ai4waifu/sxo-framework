import { reportDialectFeatures } from './report-features.js';

type DialectId = 'mathematica' | 'matlab';

function isDialectId(value: string | undefined): value is DialectId {
    return value === 'mathematica' || value === 'matlab';
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

process.exitCode = await main();
