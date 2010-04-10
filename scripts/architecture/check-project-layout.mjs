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
const NPM_MEMBER_PREFIXES = ['projects/packages/', 'projects/platforms/', 'projects/tooling/', 'projects/site/'];
const PRODUCT_PACKAGE_DIRS = [
    'projects/packages/sxo-core',
    'projects/packages/sxo-lite',
    'projects/packages/sxo',
    'projects/packages/sxo-mathematica',
    'projects/packages/sxo-matlab',
    'projects/packages/sxo-simple-math',
    'projects/packages/sxo-pari-gp',
];

/** Dialect product packages that must declare a self-owned Feature Matrix. */
const DIALECT_FEATURE_MATRIX_PACKAGES = ['projects/packages/sxo-mathematica', 'projects/packages/sxo-matlab', 'projects/packages/sxo-pari-gp'];

const RUST_SCAN_ROOTS = ['projects/dialects', 'projects/bindings', 'projects/adapters'];

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

function* walkFiles(absDir, exts) {
    if (!fs.existsSync(absDir)) return;
    for (const ent of fs.readdirSync(absDir, { withFileTypes: true })) {
        const abs = path.join(absDir, ent.name);
        if (ent.isDirectory()) {
            if (ent.name === 'target' || ent.name === 'node_modules' || ent.name === 'dist') continue;
            yield* walkFiles(abs, exts);
        } else if (exts.some((e) => ent.name.endsWith(e))) {
            yield abs;
        }
    }
}

function relFromRoot(abs) {
    return path.relative(ROOT, abs).replace(/\\/g, '/');
}

function scanForbiddenRustTokens(findings) {
    const patterns = [
        { re: /\bDialect::Auto\b/, label: 'Dialect::Auto' },
        { re: /\bdetect_dialect\b/, label: 'detect_dialect' },
        { re: /\bSxoEngine\b/, label: 'SxoEngine' },
        { re: /\bathena-engine\b/, label: 'athena-engine dependency token' },
        { re: /\bathena_engine::/, label: 'athena_engine:: import' },
    ];
    for (const root of RUST_SCAN_ROOTS) {
        const absRoot = path.join(ROOT, root);
        for (const file of walkFiles(absRoot, ['.rs', '.toml'])) {
            const text = fs.readFileSync(file, 'utf8');
            const rel = relFromRoot(file);
            for (const { re, label } of patterns) {
                if (re.test(text)) {
                    findings.push({ level: 'error', message: `forbidden ${label} in ${rel}` });
                }
            }
        }
    }
}

function scanDialectCrateCrossDeps(findings) {
    const dialectDirs = [
        'projects/dialects/sxo-dialect-mathematica',
        'projects/dialects/sxo-dialect-matlab',
        'projects/dialects/sxo-dialect-simple-math',
    ];
    const dialectCrates = new Set(['sxo-dialect-mathematica', 'sxo-dialect-matlab', 'sxo-dialect-simple-math']);
    for (const dir of dialectDirs) {
        const cargoRel = path.join(dir, 'Cargo.toml');
        if (!exists(cargoRel)) continue;
        const toml = readText(cargoRel);
        for (const other of dialectCrates) {
            if (dir.endsWith(other)) continue;
            if (new RegExp(`\\b${other}\\b`).test(toml)) {
                findings.push({
                    level: 'error',
                    message: `${cargoRel} must not depend on ${other}`,
                });
            }
        }
    }
}

function scanBugsUrl(findings) {
    for (const globPat of listPnpmGlobs()) {
        for (const member of expandWorkspaceGlob(globPat)) {
            const pkgRel = path.join(member, 'package.json');
            if (!exists(pkgRel)) continue;
            const pkg = JSON.parse(readText(pkgRel));
            const bugs = pkg.bugs?.url;
            if (typeof bugs === 'string' && bugs.includes('/issues')) {
                findings.push({
                    level: 'error',
                    message: `${pkgRel} bugs.url must not point at GitHub Issues (use Discussions bugs)`,
                });
            }
        }
    }
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

        if (exists('projects/runtimes/sxo-pari-gp')) {
            findings.push({
                level: 'error',
                message: '`projects/runtimes/sxo-pari-gp` is retired; use `projects/packages/sxo-pari-gp`',
            });
        }
        if (!exists('projects/packages/sxo-pari-gp/package.json')) {
            findings.push({
                level: 'error',
                message: '`projects/packages/sxo-pari-gp` product package is required (Living 03/05/13)',
            });
        }
        for (const dir of DIALECT_FEATURE_MATRIX_PACKAGES) {
            if (!exists(`${dir}/package.json`)) continue;
            try {
                const pkg = JSON.parse(readText(`${dir}/package.json`));
                const rel = typeof pkg?.sxo?.featureMatrix === 'string' ? pkg.sxo.featureMatrix.trim() : '';
                if (!rel) {
                    findings.push({
                        level: 'error',
                        message: `${dir}/package.json must declare sxo.featureMatrix (dialect-owned Feature Matrix)`,
                    });
                    continue;
                }
                const matrixPath = path.posix.join(dir, rel.replace(/\\/g, '/'));
                if (!exists(matrixPath)) {
                    findings.push({
                        level: 'error',
                        message: `${dir} sxo.featureMatrix points to missing file: ${rel}`,
                    });
                }
            } catch (err) {
                findings.push({
                    level: 'error',
                    message: `${dir}/package.json is not valid JSON (${err instanceof Error ? err.message : String(err)})`,
                });
            }
        }

        if (exists('projects/bindings/sxo-engine') || exists('projects/packages/sxo-engine')) {
            findings.push({ level: 'error', message: 'sxo-engine path must not exist' });
        }

        scanForbiddenRustTokens(findings);
        scanDialectCrateCrossDeps(findings);
        scanBugsUrl(findings);
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
    console.log(`check-project-layout: ${strict ? 'strict' : 'inventory'} ok (${findings.length} finding(s))`);
}

main();
