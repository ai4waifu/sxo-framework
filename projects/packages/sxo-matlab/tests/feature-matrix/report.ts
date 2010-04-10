import { listCasesByFlags, toConsoleRows, toMarkdownTable, validateFeatureMatrix } from '@sxo/harness';
import { featureMatrix } from './index.js';

const mode = process.argv[2] ?? 'markdown';
const flagArg = process.argv.find((a) => a.startsWith('--flag='))?.slice('--flag='.length);
const validation = validateFeatureMatrix(featureMatrix);
if (!validation.ok) {
    const detail = validation.issues.map((i) => `  - ${i.message}`).join('\n');
    console.error(`Invalid matlab feature matrix:\n${detail}`);
    process.exitCode = 1;
} else if (mode === 'wrongs' || mode === 'wrong' || flagArg) {
    const want = flagArg
        ? flagArg
              .split(',')
              .map((f) => f.trim())
              .filter(Boolean)
        : ['wrong'];
    const rows = listCasesByFlags(featureMatrix, want);
    console.table(rows);
    console.error(`flagged cases: ${rows.length} (flags=${want.join(',')})`);
} else if (mode === 'table') {
    console.table(toConsoleRows(featureMatrix));
} else {
    process.stdout.write(toMarkdownTable(featureMatrix));
}
