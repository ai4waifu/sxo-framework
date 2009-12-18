/**
 * Project-layout architecture gate (Living 16 §12).
 *
 * Reports current layout against the move manifest and post-migration rules.
 *
 * Usage:
 *   node scripts/architecture/check-project-layout.mjs
 *   node scripts/architecture/check-project-layout.mjs --strict
 *
 * Default mode inventories the current tree (exit 0 when layout is consistent
 * with either pre-move sources or post-move targets).
 * `--strict` requires the post-migration layout and fails if compilers/runtimes remain.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const MANIFEST_PATH = path.join(ROOT, 'scripts/architecture/project-layout.v1.json');

function fail(msg) {
    console.error(`check-project-layout: ${msg}`);
    process.exit(1);
}

function exists(rel) {
    return fs.existsSync(path.join(ROOT, rel));
}

function main() {
    const strict = process.argv.includes('--strict');
    const manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8'));
    /** @type {{ level: 'info' | 'warn' | 'error', message: string }[]} */
    const findings = [];

    let sourcesPresent = 0;
    let targetsPresent = 0;
    let mixed = 0;

    for (const entry of manifest.entries) {
        const fromOk = exists(entry.from);
        const toOk = exists(entry.to);
        if (fromOk && !toOk) {
            sourcesPresent += 1;
            findings.push({ level: 'info', message: `pre-move source present: ${entry.from}` });
        } else if (!fromOk && toOk) {
            targetsPresent += 1;
            findings.push({ level: 'info', message: `post-move target present: ${entry.to}` });
        } else if (fromOk && toOk) {
            mixed += 1;
            findings.push({
                level: 'error',
                message: `both source and target exist: ${entry.from} and ${entry.to}`,
            });
        } else if (entry.required !== false) {
            findings.push({
                level: 'error',
                message: `required path missing on both sides: ${entry.from} / ${entry.to}`,
            });
        }
    }

    for (const ex of manifest.excludedFromMove ?? []) {
        findings.push({
            level: 'info',
            message: `excluded from move: ${ex.path} (${ex.reason})`,
        });
    }

    const compilers = exists('projects/compilers');
    const runtimes = exists('projects/runtimes');
    if (strict) {
        if (compilers) findings.push({ level: 'error', message: 'projects/compilers must not exist after migration' });
        if (runtimes) findings.push({ level: 'error', message: 'projects/runtimes must not exist after migration' });
        if (sourcesPresent > 0) {
            findings.push({
                level: 'error',
                message: `strict mode: ${sourcesPresent} pre-move sources still present`,
            });
        }
        if (mixed > 0) {
            findings.push({ level: 'error', message: `strict mode: ${mixed} mixed source/target pairs` });
        }
    } else {
        findings.push({
            level: 'info',
            message: `inventory: sources=${sourcesPresent} targets=${targetsPresent} mixed=${mixed} compilers=${compilers} runtimes=${runtimes}`,
        });
    }

    for (const f of findings) {
        const tag = f.level.toUpperCase();
        console.log(`[${tag}] ${f.message}`);
    }

    const errors = findings.filter((f) => f.level === 'error');
    if (errors.length) {
        fail(`${errors.length} error(s)`);
    }
    console.log(
        `check-project-layout: ${strict ? 'strict' : 'inventory'} ok (${findings.length} finding(s))`,
    );
}

main();
