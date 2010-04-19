/**
 * MATLAB Form boundary gate (Living 14 / 17).
 *
 * - `parse.rs` must not push arena terms directly (only via `form_to_term`).
 * - `lower_term_request` must not exist (hosts retain Form; no Term→Form reconstruct).
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
const NAPI_SESSION = path.join(ROOT, 'projects/bindings/sxo-napi/src/session/mod.rs');
const WASM_SESSION = path.join(ROOT, 'projects/bindings/sxo-lite-wasm/src/session/mod.rs');

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
        if (c === '"' || c === "'") {
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

const lowerCode = stripRustNoise(fs.readFileSync(LOWER, 'utf8'));
if (/\blower_term_request\b/.test(lowerCode)) {
    fail('lower_term_request must be deleted; hosts retain MatlabForm');
}
if (/\bfn term_to_form\b/.test(lowerCode)) {
    fail('term_to_form reconstruct must be deleted; do not reverse arena Terms into Form');
}

for (const [label, file] of [
    ['sxo-napi session', NAPI_SESSION],
    ['sxo-lite-wasm session', WASM_SESSION],
]) {
    if (!fs.existsSync(file)) fail(`missing ${label}`);
    const code = stripRustNoise(fs.readFileSync(file, 'utf8'));
    if (/\blower_term_request\b/.test(code)) {
        fail(`${label}: must not call lower_term_request`);
    }
}

console.log('check-matlab-form-boundary: ok');
