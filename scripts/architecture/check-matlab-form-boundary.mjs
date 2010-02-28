/**
 * MATLAB Form boundary gate (Living 14).
 *
 * - `parse.rs` must not push arena terms directly (only via `form_to_term`).
 * - `lower_term_request` must keep the transitional "do not add new heads" banner.
 *
 * Usage:
 *   node scripts/architecture/check-matlab-form-boundary.mjs
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const PARSE = path.join(ROOT, 'projects/dialects/sxo-dialect-matlab/src/parse.rs');
const LOWER = path.join(ROOT, 'projects/dialects/sxo-dialect-matlab/src/lower.rs');

function fail(msg) {
    console.error(`check-matlab-form-boundary: ${msg}`);
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

if (!fs.existsSync(PARSE) || !fs.existsSync(LOWER)) {
    fail('matlab dialect sources missing');
}

const parseCode = stripRustNoise(fs.readFileSync(PARSE, 'utf8'));
const forbiddenParse = [
    /\bpush_semantic\b/,
    /\bpush_matlab_call\b/,
    /\bpush_list\b/,
    /\bpush_symbol_name\b/,
    /\bpush_bool\b/,
    /\bpush_null\b/,
    /\barena\.push\b/,
];
for (const re of forbiddenParse) {
    if (re.test(parseCode)) {
        fail(`parse.rs must not build arena terms directly (${re}); use MatlabForm then form_to_term`);
    }
}
if (!/\bform_to_term\b/.test(parseCode)) {
    fail('parse.rs must call form_to_term for the TermId bridge');
}

const lowerText = fs.readFileSync(LOWER, 'utf8');
if (!/Do \*\*not\*\* add new request-shaped heads here/.test(lowerText)) {
    fail('lower_term_request must keep the transitional "Do **not** add new request-shaped heads here" banner');
}

console.log('check-matlab-form-boundary: ok');
