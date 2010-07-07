/**
 * Structured execution gate (SXO Living 17 / R-2.11).
 *
 * Evaluate / fork paths must not serialize through dialect display text.
 *
 * Usage:
 *   node scripts/architecture/check-structured-execute.mjs
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

const HANDLE_TARGETS = ['projects/bindings/sxo-napi/src/handles/mod.rs', 'projects/bindings/sxo-lite-wasm/src/handles/mod.rs'];

const SESSION_TARGETS = ['projects/bindings/sxo-napi/src/session/mod.rs', 'projects/bindings/sxo-lite-wasm/src/session/mod.rs'];

function fail(msg) {
    console.error(`check-structured-execute: ${msg}`);
    process.exit(1);
}

function stripRustComments(src) {
    return src.replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ' ')).replace(/\/\/[^\n]*/g, '');
}

for (const rel of HANDLE_TARGETS) {
    const file = path.join(ROOT, rel);
    if (!fs.existsSync(file)) fail(`missing ${rel}`);
    const code = stripRustComments(fs.readFileSync(file, 'utf8'));
    if (/\bfork_expression\b/.test(code)) {
        fail(`${rel}: remove fork_expression (display-text / Form round-trip fork)`);
    }
    // evaluate body must not call display renderers as execution input
    const evaluateMatch = code.match(/fn evaluate\([\s\S]*?\n\s*}/);
    if (!evaluateMatch) fail(`${rel}: evaluate method missing`);
    const body = evaluateMatch[0];
    if (/\brender_as_matlab\b/.test(body) || /\brender_as_wolfram\b/.test(body)) {
        fail(`${rel}: evaluate must not call render_as_matlab / render_as_wolfram`);
    }
    if (/\bevaluate_matlab\s*\(\s*&?\s*text\b/.test(body)) {
        fail(`${rel}: evaluate must not re-parse MATLAB display text`);
    }
    if (/\blower_term_request\b/.test(code)) {
        fail(`${rel}: must not call lower_term_request`);
    }
    if (!/\bHeldForm\b/.test(code)) {
        fail(`${rel}: Expression must retain HeldForm on parse objects`);
    }
}

for (const rel of SESSION_TARGETS) {
    const file = path.join(ROOT, rel);
    if (!fs.existsSync(file)) fail(`missing ${rel}`);
    const code = stripRustComments(fs.readFileSync(file, 'utf8'));
    for (const name of ['evaluate_matlab_form', 'evaluate_wolfram_form']) {
        const re = new RegExp(`fn ${name}\\([\\s\\S]*?\\n\\s*\\}`);
        const m = code.match(re);
        if (!m) fail(`${rel}: ${name} missing`);
        const body = m[0];
        if (/\bform_to_term\b/.test(body) || /\blower_wexpr\b/.test(body)) {
            fail(`${rel}: ${name} must not eager-fallback via form_to_term / lower_wexpr`);
        }
    }
}

console.log('check-structured-execute: ok');
