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
 * `--strict` requires the post-migration layout and fails on Living 16 layout
 * membership / forbidden-path rules.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const MANIFEST_PATH = path.join(ROOT, 'scripts/architecture/project-layout.v1.json');

const RUST_MEMBER_PREFIXES = ['projects/dialects/', 'projects/bindings/', 'projects/adapters/'];
const NPM_MEMBER_PREFIXES = [
    'projects/packages/',
    'projects/platforms/',
    'projects/tooling/',
    'projects/site/',
];
const PRODUCT_PACKAGE_DIRS = [
    'projects/packages/sxo-core',
    'projects/packages/sxo-lite',
    'projects/packages/sxo',
    'projects/packages/sxo-mathematica',
    'projects/packages/sxo-matlab',
    'projects/packages/sxo-simple-math',
];

function fail(msg) {
    console.error(`check-project-layout: ${msg}`);
    process.exit(1);
}

function exists(rel) {
    return fs.existsSync(path.join(ROOT, rel));
}

function readText(rel) {
    return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

function parseTomlStringArray(toml, key) {
    const re = new RegExp(`${key}\\s*=\\s*\\[([\\s\\S]*?)\\]`);
    const m = toml.match(re);
    if (!m) return [];
    return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1].replace(/\\/g, '/'));
}

function listPnpmGlobs() {
    const yaml = readText('pnpm-workspace.yaml');
    return [...yaml.matchAll(/^\s*-\s*'([^']+)'/gm)].map((m) => m[1]);
}

function expandWorkspaceGlob(globPat) {
    // Only supports `projects/<segment>/*` and `projects/<a>/<b>/*`.
    const parts = globPat.split('/');
    if (parts.at(-1) !== '*') {
        const abs = path.join(ROOT, globPat);
        return fs.existsSync(abs) ? [globPat.replace(/\\/g, '/')] : [];
    }
    const parent = parts.slice(0, -1).join('/');
    const absParent = path.join(ROOT, parent);
    if (!fs.existsSync(absParent)) return [];
    return fs
        .readdirSync(absParent, { withFileTypes: true })
        .filter((d) => d.isDirectory() && fs.existsSync(path.join(absParent, d.name, 'package.json')))
        .map((d) => `${parent}/${d.name}`.replace(/\\/g, '/'));
}

function packageJsonDeps(relDir) {
    const pkg = JSON.parse(readText(path.join(relDir, 'package.json')));
    return {
        dependencies: pkg.dependencies ?? {},
        optionalDependencies: pkg.optionalDependencies ?? {},
        peerDependencies: pkg.peerDependencies ?? {},
        devDependencies: pkg.devDependencies ?? {},
    };
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

        const cargoToml = readText('Cargo.toml');
        for (const member of parseTomlStringArray(cargoToml, 'members')) {
            const ok = RUST_MEMBER_PREFIXES.some((p) => member.startsWith(p));
            if (!ok) {
                findings.push({
                    level: 'error',
                    message: `Cargo member outside dialects/bindings/adapters: ${member}`,
                });
            }
        }

        for (const globPat of listPnpmGlobs()) {
            for (const member of expandWorkspaceGlob(globPat)) {
                const ok = NPM_MEMBER_PREFIXES.some((p) => member.startsWith(p));
                if (!ok) {
                    findings.push({
                        level: 'error',
                        message: `pnpm workspace member outside packages/platforms/tooling/site: ${member}`,
                    });
                }
            }
        }

        for (const dir of PRODUCT_PACKAGE_DIRS) {
            if (!exists(path.join(dir, 'package.json'))) continue;
            const deps = packageJsonDeps(dir);
            for (const section of ['dependencies', 'optionalDependencies', 'peerDependencies']) {
                if (deps[section]['@sxo/harness']) {
                    findings.push({
                        level: 'error',
                        message: `${dir}/package.json ${section} must not include @sxo/harness`,
                    });
                }
            }
        }

        if (exists('projects/runtimes/sxo-pari-gp') || exists('projects/packages/sxo-pari-gp')) {
            findings.push({
                level: 'error',
                message: 'feasibility placeholder @sxo/pari-gp must not exist',
            });
        }

        if (exists('projects/bindings/sxo-engine') || exists('projects/packages/sxo-engine')) {
            findings.push({ level: 'error', message: 'sxo-engine path must not exist' });
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
