/**
 * GitHub Actions: publish real npm packages (not placeholder stubs).
 *
 * - tag vX.Y.Z → version X.Y.Z
 * - Idempotent: skip when version already on registry
 * - No NPM_TOKEN; OIDC Trusted Publisher (permissions.id-token: write)
 * - Contract: file=publish-npm.yml env=NPM_PUBLISH repo=ai4waifu/sxo-framework
 *
 * Prereq: native artifacts in SXO_NATIVE_ARTIFACTS (see publish-npm.yml).
 */

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const REPO_URL = 'git+https://github.com/ai4waifu/sxo-framework.git';

const NATIVE_PLATFORMS = [
    { short: 'win32-x64', triple: 'win32-x64-msvc', os: ['win32'], cpu: ['x64'] },
    { short: 'darwin-x64', triple: 'darwin-x64', os: ['darwin'], cpu: ['x64'] },
    { short: 'darwin-arm64', triple: 'darwin-arm64', os: ['darwin'], cpu: ['arm64'] },
    { short: 'linux-x64', triple: 'linux-x64-gnu', os: ['linux'], cpu: ['x64'] },
];

/** Packages that load the optional native addon at runtime. */
const NATIVE_CONSUMERS = new Set(['@sxo/core', '@sxo/mathematica', '@sxo/matlab']);

/** @type {{ dir: string, publishName?: string }[]} */
const JS_PACKAGES = [
    { dir: 'projects/runtimes/sxo-lite-unknown-wasm32', publishName: '@sxo/lite-unknown-wasm32' },
    { dir: 'projects/runtimes/sxo-lite', publishName: '@sxo/lite' },
    { dir: 'projects/runtimes/sxo-core', publishName: '@sxo/core' },
    { dir: 'projects/runtimes/sxo-simple-math', publishName: '@sxo/simple-math' },
    { dir: 'projects/runtimes/sxo-mathematica', publishName: '@sxo/mathematica' },
    { dir: 'projects/runtimes/sxo-matlab', publishName: '@sxo/matlab' },
    { dir: 'projects/runtimes/sxo', publishName: '@sxo/sxo' },
];

function fail(msg) {
    console.error(`ci-publish-npm: ${msg}`);
    process.exit(1);
}

function run(cmd, args, opts = {}) {
    const r = spawnSync(cmd, args, {
        cwd: opts.cwd ?? ROOT,
        encoding: 'utf8',
        shell: process.platform === 'win32',
        env: opts.env ?? process.env,
        stdio: opts.stdio ?? 'pipe',
    });
    return {
        status: r.status ?? 1,
        stdout: String(r.stdout ?? '').trim(),
        stderr: String(r.stderr ?? '').trim(),
    };
}

function resolveVersion() {
    const fromArg = process.argv.find((a) => a.startsWith('--version='))?.slice('--version='.length);
    if (fromArg) return fromArg.replace(/^v/, '');
    const ref = process.env.GITHUB_REF ?? '';
    const m = ref.match(/^refs\/tags\/(?:placeholder-)?v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/);
    if (m) return m[1];
    fail('need --version=X.Y.Z or GITHUB_REF=refs/tags/vX.Y.Z');
}

function readJson(p) {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function writeJson(p, obj) {
    fs.writeFileSync(p, `${JSON.stringify(obj, null, 4)}\n`);
}

function copyTree(src, dest, filter) {
    fs.mkdirSync(dest, { recursive: true });
    for (const name of fs.readdirSync(src)) {
        if (name === 'node_modules' || name === '.git') continue;
        const from = path.join(src, name);
        const to = path.join(dest, name);
        const st = fs.statSync(from);
        if (st.isDirectory()) {
            if (filter && !filter(from, true)) continue;
            copyTree(from, to, filter);
        } else {
            if (filter && !filter(from, false)) continue;
            fs.copyFileSync(from, to);
        }
    }
}

function rewriteWorkspaceDeps(deps, version) {
    if (!deps) return deps;
    /** @type {Record<string, string>} */
    const out = {};
    for (const [k, v] of Object.entries(deps)) {
        if (typeof v === 'string' && (v.startsWith('workspace:') || v === '*')) {
            out[k] = version;
        } else {
            out[k] = v;
        }
    }
    return out;
}

function rewriteDepsField(pkg, version) {
    for (const field of ['dependencies', 'optionalDependencies', 'peerDependencies']) {
        if (pkg[field]) pkg[field] = rewriteWorkspaceDeps(pkg[field], version);
    }
    return pkg;
}

function isAlreadyPublished(blob) {
    return /cannot publish over existing|EPUBLISHCONFLICT|previously published versions|version already exists|cannot publish.*same version|you cannot publish over/i.test(
        blob,
    );
}

function isAuthFailure(blob) {
    return /ENEEDAUTH|Unable to authenticate|not authorized|OIDC|trusted publisher|two-factor|need to be logged|login|identity token|do not have permission to access it|Access token expired or revoked/i.test(
        blob,
    );
}

function isMissingPackage(blob) {
    if (isAuthFailure(blob)) return false;
    return /Package not found|does not exist on the registry|cannot publish.*before creating|This package has not been created|is not in this registry/i.test(
        blob,
    );
}

function versionExists(name, version) {
    const r = run('npm', ['view', `${name}@${version}`, 'version']);
    return r.status === 0 && r.stdout === version;
}

function npmPublish(stagingDir, name, version) {
    const args = ['publish', '--access', 'public'];
    console.log(`\n=== ${name}@${version} npm ${args.join(' ')} ===`);
    const r = run('npm', args, { cwd: stagingDir });
    if (r.stdout) process.stdout.write(`${r.stdout}\n`);
    if (r.stderr) process.stderr.write(`${r.stderr}\n`);
    const blob = `${r.stdout}\n${r.stderr}`;
    if (r.status === 0) return 'published';
    if (isAlreadyPublished(blob) || versionExists(name, version)) return 'exists';
    if (isAuthFailure(blob)) return 'auth';
    if (isMissingPackage(blob)) return 'missing';
    if (versionExists(name, version)) return 'exists';
    console.error(blob.slice(0, 1200));
    return 'other';
}

function trustedPublisherHint() {
    return 'Add Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=ai4waifu/sxo-framework';
}

function publishNative(version, artifactsRoot) {
    let published = 0;
    let skipped = 0;
    for (const plat of NATIVE_PLATFORMS) {
        const name = `@sxo/sxo-${plat.short}`;
        const artDir = path.join(artifactsRoot, plat.short);
        if (!fs.existsSync(artDir)) {
            console.log(` · ${name} no artifact (${plat.short}) — skip`);
            skipped += 1;
            continue;
        }
        if (versionExists(name, version)) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }
        const stage = path.join(os.tmpdir(), `sxo-pub-native-${plat.short}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        fs.mkdirSync(stage, { recursive: true });
        for (const f of fs.readdirSync(artDir)) {
            fs.copyFileSync(path.join(artDir, f), path.join(stage, f));
        }
        const want = `sxo.${plat.triple}.node`;
        writeJson(path.join(stage, 'package.json'), {
            name,
            version,
            description: `SXO native N-API addon (${plat.short})`,
            license: 'Apache-2.0',
            private: false,
            os: plat.os,
            cpu: plat.cpu,
            main: want,
            files: [want, 'README.md'],
            publishConfig: { access: 'public' },
            repository: { type: 'git', url: REPO_URL },
        });
        for (const f of fs.readdirSync(stage)) {
            if (f.endsWith('.node') && f !== want) fs.unlinkSync(path.join(stage, f));
        }
        const plain = path.join(stage, 'sxo.node');
        if (!fs.existsSync(path.join(stage, want)) && fs.existsSync(plain)) {
            fs.renameSync(plain, path.join(stage, want));
        }
        if (!fs.existsSync(path.join(stage, want))) {
            fail(`${name}: staged artifact missing ${want}`);
        }
        fs.writeFileSync(
            path.join(stage, 'README.md'),
            `# ${name}\n\nOptional native binary for \`@sxo/core\` (${plat.short} / ${plat.triple}).\n`,
        );
        const outcome = npmPublish(stage, name, version);
        if (outcome === 'published') published += 1;
        else if (outcome === 'exists') {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
        } else if (outcome === 'auth') {
            fail(`OIDC/auth failed for ${name}. ${trustedPublisherHint()}`);
        } else fail(`publish failed for ${name}`);
    }
    return { published, skipped };
}

function publishJs(version) {
    let published = 0;
    let skipped = 0;
    const optionalNatives = Object.fromEntries(NATIVE_PLATFORMS.map((p) => [`@sxo/sxo-${p.short}`, version]));

    for (const spec of JS_PACKAGES) {
        const abs = path.join(ROOT, spec.dir);
        if (!fs.existsSync(abs)) fail(`missing package dir ${spec.dir}`);
        const raw = readJson(path.join(abs, 'package.json'));
        const name = spec.publishName ?? raw.name;
        if (!name) fail(`no name for ${spec.dir}`);

        if (versionExists(name, version)) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }

        const stage = path.join(os.tmpdir(), `sxo-pub-js-${name.replace(/[/@]/g, '-')}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        fs.mkdirSync(stage, { recursive: true });

        const files = Array.isArray(raw.files) && raw.files.length ? raw.files : null;
        if (files) {
            for (const f of files) {
                const from = path.join(abs, f);
                if (!fs.existsSync(from)) continue;
                const st = fs.statSync(from);
                const to = path.join(stage, f);
                if (st.isDirectory()) copyTree(from, to);
                else {
                    fs.mkdirSync(path.dirname(to), { recursive: true });
                    fs.copyFileSync(from, to);
                }
            }
            for (const extra of ['package.json', 'README.md', 'LICENSE', 'bin', 'locales']) {
                const from = path.join(abs, extra);
                if (!fs.existsSync(from)) continue;
                const to = path.join(stage, extra);
                if (fs.statSync(from).isDirectory()) copyTree(from, to);
                else fs.copyFileSync(from, to);
            }
        } else {
            copyTree(abs, stage, (p) => {
                const rel = path.relative(abs, p);
                if (rel.includes('node_modules') || rel.includes('tests') || rel.endsWith('.node')) return false;
                return true;
            });
        }

        if (name === '@sxo/sxo') {
            for (const f of fs.readdirSync(stage)) {
                if (f.endsWith('.node')) fs.unlinkSync(path.join(stage, f));
            }
        }

        const pkg = rewriteDepsField({ ...raw }, version);
        pkg.name = name;
        pkg.version = version;
        delete pkg.private;
        pkg.publishConfig = { ...(pkg.publishConfig ?? {}), access: 'public' };

        if (name === '@sxo/lite-unknown-wasm32') {
            const wasm = path.join(stage, 'lib/sxo_lite_bg.wasm');
            const entry = path.join(stage, 'dist/index.js');
            if (!fs.existsSync(wasm)) {
                fail(`${name}: missing lib/sxo_lite_bg.wasm — run pnpm build:lite-wasm before publish`);
            }
            if (!fs.existsSync(entry)) {
                fail(`${name}: missing dist/index.js — run pnpm build:lite-wasm before publish`);
            }
        }
        if (pkg.scripts) {
            delete pkg.scripts.prepack;
            delete pkg.scripts.prepare;
            if (Object.keys(pkg.scripts).length === 0) delete pkg.scripts;
        }
        if (!pkg.repository) {
            pkg.repository = { type: 'git', url: REPO_URL };
        }
        if (NATIVE_CONSUMERS.has(name)) {
            pkg.optionalDependencies = { ...(pkg.optionalDependencies ?? {}), ...optionalNatives };
        }
        delete pkg.devDependencies;
        writeJson(path.join(stage, 'package.json'), pkg);

        if (!fs.existsSync(path.join(stage, 'README.md'))) {
            fs.writeFileSync(path.join(stage, 'README.md'), `# ${name}\n\nSXO package ${version}.\n`);
        }

        const outcome = npmPublish(stage, name, version);
        if (outcome === 'published') published += 1;
        else if (outcome === 'exists') {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
        } else if (outcome === 'auth') {
            fail(`OIDC/auth failed for ${name}. ${trustedPublisherHint()}`);
        } else if (outcome === 'missing') {
            fail(`${name} is not on the registry yet. Run pnpm placeholder:publish then pnpm placeholder:trust, then retry.`);
        } else fail(`publish failed for ${name}`);
    }
    return { published, skipped };
}

const version = resolveVersion();
console.log(`ci-publish-npm: version=${version}`);
console.log(` GITHUB_REF=${process.env.GITHUB_REF ?? '(none)'}`);
console.log(' Trusted Publisher contract: publish-npm.yml + env NPM_PUBLISH + repo ai4waifu/sxo-framework\n');

delete process.env.NODE_AUTH_TOKEN;
delete process.env.NPM_TOKEN;

const artifactsRoot = process.env.SXO_NATIVE_ARTIFACTS || path.join(ROOT, 'dist', 'native-flat');

const native = publishNative(version, artifactsRoot);
const js = publishJs(version);

console.log(
    `\nci-publish-npm: done (native published=${native.published} skipped=${native.skipped}; js published=${js.published} skipped=${js.skipped})`,
);
