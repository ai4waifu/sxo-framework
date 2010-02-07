/**
 * Forbid new GreenTree / CST layout walks in SXO dialect sources (Living 14).
 *
 * Typed AST consumption is allowed. TokenType operator identity is allowed.
 * Direct GreenTree / language ElementType layout matching is not.
 *
 * Usage:
 *   node scripts/architecture/check-no-greentree-layout.mjs
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const SCAN_ROOT = path.join(ROOT, 'projects/dialects');

const FORBIDDEN = [
    { id: 'GreenTree', re: /\bGreenTree\b/ },
    { id: 'WolframElementType', re: /\bWolframElementType\b/ },
    { id: 'MatlabElementType', re: /\bMatlabElementType\b/ },
    { id: 'green_node', re: /\bgreen_node\b/ },
    { id: 'SyntaxKind walk', re: /\.children\s*\(/ },
];

function stripRustNoise(src) {
    // Drop block comments, line comments, and string/char literals so doc mentions do not trip the gate.
    let out = '';
    let i = 0;
    while (i < src.length) {
        if (src.startsWith('/*', i)) {
            const end = src.indexOf('*/', i + 2);
            const block = end < 0 ? src.slice(i) : src.slice(i, end + 2);
            out += block.replace(/[^\n]/g, ' ');
            i = end < 0 ? src.length : end + 2;
            continue;
        }
        if (src.startsWith('//', i)) {
            const end = src.indexOf('\n', i);
            if (end < 0) {
                i = src.length;
            } else {
                out += '\n';
                i = end + 1;
            }
            continue;
        }
        const c = src[i];
        if (c === '"' || c === '\'') {
            const quote = c;
            i += 1;
            while (i < src.length) {
                if (src[i] === '\\') {
                    i += 2;
                    continue;
                }
                if (src[i] === quote) {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out += '""';
            continue;
        }
        out += c;
        i += 1;
    }
    return out;
}

function walkRsFiles(dir, acc = []) {
    if (!fs.existsSync(dir)) return acc;
    for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
        const abs = path.join(dir, ent.name);
        if (ent.isDirectory()) walkRsFiles(abs, acc);
        else if (ent.isFile() && ent.name.endsWith('.rs')) acc.push(abs);
    }
    return acc;
}

const findings = [];
for (const file of walkRsFiles(SCAN_ROOT)) {
    const rel = path.relative(ROOT, file).replace(/\\/g, '/');
    const raw = fs.readFileSync(file, 'utf8');
    const code = stripRustNoise(raw);
    const lines = code.split(/\r?\n/);
    for (let li = 0; li < lines.length; li += 1) {
        const line = lines[li];
        for (const rule of FORBIDDEN) {
            if (rule.re.test(line)) {
                findings.push(`${rel}:${li + 1}: forbidden CST layout dependency (${rule.id})`);
            }
        }
    }
}

if (findings.length > 0) {
    console.error('check-no-greentree-layout: Living 14 forbids new GreenTree / ElementType layout walks in dialects.');
    for (const f of findings) console.error(`  ${f}`);
    process.exit(1);
}

console.log('check-no-greentree-layout: ok (no dialect GreenTree / ElementType layout walks)');
