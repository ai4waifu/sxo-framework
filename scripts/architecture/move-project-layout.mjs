/**
 * Deterministic project-layout mover (Living 16).
 *
 * Reads scripts/architecture/project-layout.v1.json.
 * Moves registered paths only. Never rewrites source contents.
 *
 * Usage:
 *   node scripts/architecture/move-project-layout.mjs --check
 *   node scripts/architecture/move-project-layout.mjs --apply
 *   node scripts/architecture/move-project-layout.mjs --apply --report out.json
 */

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const MANIFEST_PATH = path.join(ROOT, 'scripts/architecture/project-layout.v1.json');

const SKIP_DIR_NAMES = new Set(['node_modules', 'dist', 'target', '.git']);

function fail(msg) {
    console.error(`move-project-layout: ${msg}`);
    process.exit(1);
}

function parseArgs(argv) {
    const opts = { check: false, apply: false, report: null };
    for (let i = 0; i < argv.length; i += 1) {
        const a = argv[i];
        if (a === '--check') opts.check = true;
        else if (a === '--apply') opts.apply = true;
        else if (a === '--report') {
            opts.report = argv[i + 1];
            if (!opts.report) fail('--report requires a path');
            i += 1;
        } else fail(`unknown argument ${a}`);
    }
    if (opts.check === opts.apply) {
        fail('exactly one of --check or --apply is required');
    }
    return opts;
}

function assertInsideRoot(absPath, label) {
    const rel = path.relative(ROOT, absPath);
    if (rel.startsWith('..') || path.isAbsolute(rel)) {
        fail(`${label} escapes repository root: ${absPath}`);
    }
}

function listTrackedFiles(dirAbs) {
    /** @type {string[]} */
    const out = [];
    function walk(current) {
        for (const name of fs.readdirSync(current)) {
            if (SKIP_DIR_NAMES.has(name)) continue;
            const full = path.join(current, name);
            const st = fs.statSync(full);
            if (st.isDirectory()) walk(full);
            else out.push(path.relative(dirAbs, full).split(path.sep).join('/'));
        }
    }
    walk(dirAbs);
    out.sort();
    return out;
}

function loadManifest() {
    if (!fs.existsSync(MANIFEST_PATH)) fail(`missing manifest ${MANIFEST_PATH}`);
    const data = JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8'));
    if (!Array.isArray(data.entries)) fail('manifest.entries must be an array');
    return data;
}

function resolveEntry(entry) {
    if (!entry || typeof entry.from !== 'string' || typeof entry.to !== 'string') {
        fail('each entry needs string from/to');
    }
    const fromAbs = path.resolve(ROOT, entry.from);
    const toAbs = path.resolve(ROOT, entry.to);
    assertInsideRoot(fromAbs, 'from');
    assertInsideRoot(toAbs, 'to');
    return {
        from: entry.from.replace(/\\/g, '/'),
        to: entry.to.replace(/\\/g, '/'),
        kind: entry.kind ?? 'directory',
        required: entry.required !== false,
        fromAbs,
        toAbs,
    };
}

function gitMv(fromAbs, toAbs) {
    const parent = path.dirname(toAbs);
    fs.mkdirSync(parent, { recursive: true });
    const r = spawnSync('git', ['mv', fromAbs, toAbs], {
        cwd: ROOT,
        encoding: 'utf8',
        shell: false,
    });
    if (r.status !== 0) {
        fail(`git mv failed\n${r.stderr || r.stdout || ''}`.trim());
    }
}

function checkEntry(entry) {
    const fromExists = fs.existsSync(entry.fromAbs);
    const toExists = fs.existsSync(entry.toAbs);
    /** @type {{ ok: boolean, status: string, fileCount?: number, files?: string[], detail?: string }} */
    const result = {
        ok: true,
        status: 'ready',
    };

    if (!fromExists && toExists) {
        result.status = 'already_moved';
        result.fileCount = listTrackedFiles(entry.toAbs).length;
        return result;
    }
    if (!fromExists) {
        if (entry.required) {
            result.ok = false;
            result.status = 'missing_source';
            result.detail = `required source missing: ${entry.from}`;
        } else {
            result.status = 'optional_missing';
        }
        return result;
    }
    if (toExists) {
        result.ok = false;
        result.status = 'target_exists';
        result.detail = `target already exists: ${entry.to}`;
        return result;
    }

    const files = listTrackedFiles(entry.fromAbs);
    result.fileCount = files.length;
    result.files = files;
    return result;
}

function applyEntry(entry) {
    const before = checkEntry(entry);
    if (before.status === 'already_moved') {
        return {
            ok: true,
            status: 'already_moved',
            fileCount: before.fileCount ?? 0,
        };
    }
    if (!before.ok) {
        return {
            ok: false,
            status: before.status,
            detail: before.detail,
        };
    }

    const sourceFiles = before.files ?? listTrackedFiles(entry.fromAbs);
    gitMv(entry.fromAbs, entry.toAbs);

    if (!fs.existsSync(entry.toAbs)) {
        return { ok: false, status: 'move_failed', detail: 'target missing after git mv' };
    }
    if (fs.existsSync(entry.fromAbs)) {
        return { ok: false, status: 'move_failed', detail: 'source still present after git mv' };
    }

    const targetFiles = listTrackedFiles(entry.toAbs);
    const same =
        sourceFiles.length === targetFiles.length &&
        sourceFiles.every((f, i) => f === targetFiles[i]);
    if (!same) {
        return {
            ok: false,
            status: 'content_mismatch',
            detail: `file set mismatch after move (${sourceFiles.length} → ${targetFiles.length})`,
            fileCount: targetFiles.length,
        };
    }

    return {
        ok: true,
        status: 'moved',
        fileCount: targetFiles.length,
    };
}

function main() {
    const opts = parseArgs(process.argv.slice(2));
    const manifest = loadManifest();
    const report = {
        mode: opts.apply ? 'apply' : 'check',
        root: ROOT,
        manifest: path.relative(ROOT, MANIFEST_PATH).split(path.sep).join('/'),
        generatedAt: new Date().toISOString(),
        entries: [],
        excludedFromMove: manifest.excludedFromMove ?? [],
        ok: true,
    };

    for (const raw of manifest.entries) {
        const entry = resolveEntry(raw);
        const outcome = opts.apply ? applyEntry(entry) : checkEntry(entry);
        report.entries.push({
            from: entry.from,
            to: entry.to,
            kind: entry.kind,
            required: entry.required,
            ...outcome,
            files: undefined,
        });
        if (!outcome.ok) {
            report.ok = false;
            if (opts.apply) {
                writeReport(opts.report, report);
                fail(`stopped at ${entry.from} → ${entry.to}: ${outcome.status}${outcome.detail ? ` (${outcome.detail})` : ''}`);
            }
        }
        const mark = outcome.ok ? 'ok' : 'FAIL';
        console.log(`[${mark}] ${entry.from} → ${entry.to} (${outcome.status}${outcome.fileCount != null ? `, files=${outcome.fileCount}` : ''})`);
    }

    writeReport(opts.report, report);
    if (!report.ok) {
        process.exit(1);
    }
    console.log(`move-project-layout: ${opts.apply ? 'apply' : 'check'} succeeded (${report.entries.length} entries)`);
}

function writeReport(reportPath, report) {
    if (!reportPath) return;
    const abs = path.resolve(ROOT, reportPath);
    assertInsideRoot(abs, 'report');
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, `${JSON.stringify(report, null, 2)}\n`);
    console.log(`move-project-layout: wrote report ${path.relative(ROOT, abs)}`);
}

main();
