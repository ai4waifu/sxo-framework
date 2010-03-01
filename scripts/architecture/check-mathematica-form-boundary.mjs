/**
 * Mathematica Form boundary gate (Living 14 / 05).
 *
 * - `parse.rs` builds `WExpr` only (no arena Term pushes).
 * - `WExpr` must remain in this dialect crate (no re-export from sxo-types).
 *
 * Usage:
 *   node scripts/architecture/check-mathematica-form-boundary.mjs
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const PARSE = path.join(ROOT, 'projects/dialects/sxo-dialect-mathematica/src/parse.rs');
const LIB = path.join(ROOT, 'projects/dialects/sxo-dialect-mathematica/src/lib.rs');
const SXO_TYPES = path.join(ROOT, 'projects/bindings/sxo-types/src');

function fail(msg) {
    console.error(`check-mathematica-form-boundary: ${msg}`);
    process.exit(1);
}

function stripRustNoise(src) {
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
            if (end < 0) i = src.length;
            else {
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

function walkRs(dir, acc = []) {
    if (!fs.existsSync(dir)) return acc;
    for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
        const abs = path.join(dir, ent.name);
        if (ent.isDirectory()) walkRs(abs, acc);
        else if (ent.isFile() && ent.name.endsWith('.rs')) acc.push(abs);
    }
    return acc;
}

if (!fs.existsSync(PARSE) || !fs.existsSync(LIB)) {
    fail('mathematica dialect sources missing');
}

const parseCode = stripRustNoise(fs.readFileSync(PARSE, 'utf8'));
for (const re of [/\bTermId\b/, /\bpush_semantic\b/, /\barena\.push\b/, /\bpush_matlab_call\b/]) {
    if (re.test(parseCode)) {
        fail(`parse.rs must emit WExpr only (forbidden ${re})`);
    }
}
if (!/\bWExpr\b/.test(parseCode)) {
    fail('parse.rs must construct WExpr');
}

const libText = fs.readFileSync(LIB, 'utf8');
if (!/\bWExpr\b/.test(libText)) {
    fail('dialect lib must own WExpr');
}

for (const file of walkRs(SXO_TYPES)) {
    const text = fs.readFileSync(file, 'utf8');
    if (/\bWExpr\b/.test(text)) {
        fail(`WExpr must not appear in sxo-types (${path.relative(ROOT, file)})`);
    }
}

console.log('check-mathematica-form-boundary: ok');
