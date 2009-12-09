import { toConsoleRows, toMarkdownTable, validateFeatureMatrix } from '@sxo/harness';
import { featureMatrix } from './index.js';

const mode = process.argv[2] ?? 'markdown';
const validation = validateFeatureMatrix(featureMatrix);
if (!validation.ok) {
    const detail = validation.issues.map((i) => `  - ${i.message}`).join('\n');
    console.error(`Invalid mathematica feature matrix:\n${detail}`);
    process.exitCode = 1;
} else if (mode === 'table') {
    console.table(toConsoleRows(featureMatrix));
} else {
    process.stdout.write(toMarkdownTable(featureMatrix));
}
