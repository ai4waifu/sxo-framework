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

const TARGETS = [
  'projects/bindings/sxo-napi/src/handles/mod.rs',
  'projects/bindings/sxo-lite-wasm/src/handles/mod.rs',
];

function fail(msg) {
  console.error(`check-structured-execute: ${msg}`);
  process.exit(1);
}

function stripRustComments(src) {
  return src
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ' '))
    .replace(/\/\/[^\n]*/g, '');
}

for (const rel of TARGETS) {
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
}

console.log('check-structured-execute: ok');
