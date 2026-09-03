import { featureMatrix } from './matrix.js';
import { toConsoleRows, toMarkdownTable } from './report.js';

const mode = process.argv[2] ?? 'markdown';
if (mode === 'table') {
    console.table(toConsoleRows(featureMatrix));
} else {
    process.stdout.write(toMarkdownTable(featureMatrix));
}
