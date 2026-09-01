/**
 * Publish placeholder packages at 0.0.0 with Trusted Publisher (OIDC) config.
 *
 * - No NPM_TOKEN / NODE_AUTH_TOKEN (OIDC only)
 * - Creates stub packages + package-lock, then npm publish --access public
 * - Configures trustedPublisher (idempotent; skips when already set)
 * - Cache: .github/ci-cache/placeholder-publish.json (gitignored)
 * - Prefer npm view; on miss, treat matching cache as published
 * - --refresh: ignore cache and re-check registry / reconfigure trust
 * - --otp <code>: npm OTP for trust write; refreshes list and updates cache
 * - Also reads uncommitted `.env.placeholder.local` (repo root) or env:
 *     NPM_TOTP_SECRET / TOTP_SECRET  → RFC6238 TOTP (recomputed each npm call)
 *     NPM_OTP / OTP                  → 6-digit code, OR raw base32 secret if not 6 digits
 *     NPM_TOKEN / TOKEN              → access token for auth
 *   CLI flags win over env file / process.env.
 *
 * Usage:
 *   node scripts/ci/publish-placeholder.mjs
 *   node scripts/ci/publish-placeholder.mjs --refresh
 *   node scripts/ci/publish-placeholder.mjs --otp 123456
 *   # or in .env.placeholder.local:
 *   #   NPM_TOTP_SECRET=<authenticator base32>
 *   #   NPM_TOKEN=npm_xxx
 *   pnpm placeholder:trust
 */

import { spawnSync } from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/** @typedef {{ name: string, os?: string[], cpu?: string[], description?: string }} StubSpec */
/** @typedef {{ listedAt: string, configs: any[], matches: boolean, matchKind: string }} TrustCacheEntry */
/** @typedef {{ version?: string|null, versionAt?: string, trust?: TrustCacheEntry }} PkgCache */
/** @typedef {{ version: number, trustExpect: typeof TRUST, packages: Record<string, PkgCache> }} CacheFile */

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REPO_ROOT = path.resolve(ROOT, '..');
const CACHE_PATH = path.join(ROOT, '.placeholder-npm-cache.json');
const ENV_PATH = path.join(REPO_ROOT, '.env.placeholder.local');

/** Load KEY=VALUE from gitignored local env (no export, no quotes required). */
function loadLocalEnv(filePath) {
    /** @type {Record<string, string>} */
    const out = {};
    try {
        const text = fs.readFileSync(filePath, 'utf8');
        for (const raw of text.split(/\r?\n/)) {
            const line = raw.trim();
            if (!line || line.startsWith('#')) continue;
            const m = line.match(/^(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/);
            if (!m) continue;
            let v = m[2].trim();
            if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) {
                v = v.slice(1, -1);
            }
            out[m[1]] = v;
        }
    } catch {
        /* optional */
    }
    return out;
}

const localEnv = loadLocalEnv(ENV_PATH);

/** Keep in sync with scripts/ci/publish-npm.mjs (real publish package set). */
const JS_STUBS = [
    '@sxo/lite-unknown-wasm32',
    '@sxo/lite',
    '@sxo/core',
    '@sxo/simple-math',
    '@sxo/mathematica',
    '@sxo/matlab',
    '@sxo/sxo',
];
const NATIVE_STUBS = [
    { name: '@sxo/sxo-win32-x64', os: ['win32'], cpu: ['x64'] },
    { name: '@sxo/sxo-linux-x64', os: ['linux'], cpu: ['x64'] },
    { name: '@sxo/sxo-darwin-arm64', os: ['darwin'], cpu: ['arm64'] },
    { name: '@sxo/sxo-darwin-x64', os: ['darwin'], cpu: ['x64'] },
];

const VERSION = '0.0.0';

const TRUST = {
    repo: 'ai4waifu/sxo-framework',
    file: 'publish-npm.yml',
    env: 'NPM_PUBLISH',
};

const argv = process.argv.slice(2);
const positionals = argv.filter((a) => !a.startsWith('-'));
const command = positionals[0] ?? 'status';
const rest = argv.filter((a) => a !== command);

function fail(msg) {
    console.error(`placeholder: ${msg}`);
    process.exit(1);
}

/** @param {string[]} args @param {string} flag */
function takeFlag(args, flag) {
    const i = args.indexOf(flag);
    if (i >= 0 && args[i + 1] && !args[i + 1].startsWith('-')) return args[i + 1];
    const eq = args.find((a) => a.startsWith(`${flag}=`));
    if (eq) return eq.slice(flag.length + 1);
    return undefined;
}

const dryRun = rest.includes('--dry-run');
const refresh = rest.includes('--refresh');
const token = takeFlag(rest, '--token') ?? process.env.NPM_TOKEN ?? localEnv.NPM_TOKEN ?? localEnv.TOKEN;

const otpFlag = takeFlag(rest, '--otp') ?? process.env.NPM_OTP ?? localEnv.NPM_OTP ?? localEnv.OTP;
const totpSecretRaw =
    takeFlag(rest, '--totp-secret') ??
    process.env.NPM_TOTP_SECRET ??
    localEnv.NPM_TOTP_SECRET ??
    localEnv.TOTP_SECRET ??
    // Convenience: if NPM_OTP holds a base32 secret (not a 6-digit code), treat as secret.
    (otpFlag && !/^\d{6}$/.test(otpFlag.trim()) ? otpFlag : undefined);
const otpStatic = otpFlag && /^\d{6}$/.test(otpFlag.trim()) ? otpFlag.trim() : undefined;

/** @param {string} secret */
function decodeBase32(secret) {
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    const cleaned = secret.replace(/[\s=-]/g, '').toUpperCase();
    let bits = '';
    for (const ch of cleaned) {
        const v = alphabet.indexOf(ch);
        if (v < 0) fail(`invalid base32 in TOTP secret (bad char)`);
        bits += v.toString(2).padStart(5, '0');
    }
    const bytes = [];
    for (let i = 0; i + 8 <= bits.length; i += 8) {
        bytes.push(Number.parseInt(bits.slice(i, i + 8), 2));
    }
    if (!bytes.length) fail('TOTP secret decoded empty');
    return Buffer.from(bytes);
}

/** RFC 6238 TOTP (SHA-1, 30s, 6 digits). Fresh each call — trust loop spans many windows. */
function totpCode(secret, atMs = Date.now()) {
    const key = decodeBase32(secret);
    const counter = Math.floor(atMs / 1000 / 30);
    const buf = Buffer.alloc(8);
    buf.writeUInt32BE(Math.floor(counter / 0x100000000), 0);
    buf.writeUInt32BE(counter & 0xffffffff, 4);
    const hmac = crypto.createHmac('sha1', key).update(buf).digest();
    const offset = hmac[hmac.length - 1] & 0x0f;
    const code =
        ((hmac[offset] & 0x7f) << 24) | ((hmac[offset + 1] & 0xff) << 16) | ((hmac[offset + 2] & 0xff) << 8) | (hmac[offset + 3] & 0xff);
    return String(code % 1_000_000).padStart(6, '0');
}

/** @returns {string | undefined} */
function currentOtp() {
    if (totpSecretRaw) return totpCode(totpSecretRaw);
    return otpStatic;
}

if (rest.includes('--opt') || rest.some((a) => a.startsWith('--opt='))) {
    fail('--opt removed. Use --token for Access Token; --otp for 2FA code.');
}
if (rest.includes('--publish') || rest.includes('--trust')) {
    fail('flags --publish/--trust removed. Use subcommands: `publish` | `trust` | (default status).');
}

/** @type {StubSpec[]} */
const STUBS = [
    ...JS_STUBS.map((name) => ({ name })),
    ...NATIVE_STUBS.map((s) => ({
        ...s,
        description: `Optional native binary for @sxo/core (${s.name.replace(/^@sxo\//, '')}). Placeholder only.`,
    })),
];

/** @returns {CacheFile} */
function loadCache() {
    try {
        const raw = fs.readFileSync(CACHE_PATH, 'utf8');
        const data = JSON.parse(raw);
        if (!data || typeof data !== 'object') throw new Error('bad cache');
        const expect = data.trustExpect;
        const expectOk = expect && expect.repo === TRUST.repo && expect.file === TRUST.file && expect.env === TRUST.env;
        if (!expectOk) {
            return { version: 1, trustExpect: { ...TRUST }, packages: {} };
        }
        return {
            version: 1,
            trustExpect: { ...TRUST },
            packages: data.packages && typeof data.packages === 'object' ? data.packages : {},
        };
    } catch {
        return { version: 1, trustExpect: { ...TRUST }, packages: {} };
    }
}

/** @param {CacheFile} cache */
function saveCache(cache) {
    cache.trustExpect = { ...TRUST };
    cache.version = 1;
    fs.writeFileSync(CACHE_PATH, `${JSON.stringify(cache, null, 2)}\n`);
}

/**
 * @param {string[]} args
 * @param {{ cwd?: string, silent?: boolean, inherit?: boolean }} [opts]
 */
function runNpm(args, opts = {}) {
    /** @type {NodeJS.ProcessEnv} */
    const env = { ...process.env };
    let userConfig;
    if (token) {
        userConfig = path.join(os.tmpdir(), `sxo-placeholder-npmrc-${process.pid}`);
        fs.writeFileSync(userConfig, `//registry.npmjs.org/:_authToken=${token}\n`, 'utf8');
        env.NPM_CONFIG_USERCONFIG = userConfig;
        // Avoid empty NODE_AUTH_TOKEN clobbering OIDC elsewhere; only for local trust/publish stubs.
        env.NODE_AUTH_TOKEN = token;
    }
    try {
        const r = spawnSync('npm', args, {
            cwd: opts.cwd,
            encoding: 'utf8',
            shell: process.platform === 'win32',
            stdio: opts.inherit ? 'inherit' : 'pipe',
            env,
        });
        if (r.error && !opts.silent) fail(r.error.message);
        return {
            status: r.status ?? 1,
            stdout: opts.inherit ? '' : String(r.stdout ?? '').trim(),
            stderr: opts.inherit ? '' : String(r.stderr ?? '').trim(),
        };
    } finally {
        if (userConfig) {
            try {
                fs.unlinkSync(userConfig);
            } catch {
                /* ignore */
            }
        }
    }
}

function sleep(ms) {
    const end = Date.now() + ms;
    while (Date.now() < end) {
        /* rate-limit */
    }
}

function viewVersion(name) {
    const r = runNpm(['view', name, 'version'], { silent: true });
    if (r.status === 0 && r.stdout) return r.stdout;
    // Retry after short registry lag / CDN miss.
    sleep(400);
    const r2 = runNpm(['view', name, 'version'], { silent: true });
    if (r2.status === 0 && r2.stdout) return r2.stdout;
    return null;
}

/**
 * live view 优先；失败则?publish 写入?cache? * @param {CacheFile} cache
 * @param {string} name
 * @returns {{ version: string|null, source: 'live'|'cache'|'none', liveMiss: boolean }}
 */
function resolvePublishedVersion(cache, name) {
    const live = viewVersion(name);
    if (live) {
        if (!cache.packages[name]) cache.packages[name] = {};
        cache.packages[name].version = live;
        cache.packages[name].versionAt = new Date().toISOString();
        return { version: live, source: 'live', liveMiss: false };
    }
    const cached = cache.packages[name]?.version;
    if (cached) {
        return { version: cached, source: 'cache', liveMiss: true };
    }
    return { version: null, source: 'none', liveMiss: true };
}

/** npm view 短暂 miss 时的统一提示（不假装“从未发布”） */
function warnLiveViewMiss(name, detail) {
    console.log(`  ! ${name}  npm view miss (registry lag / CDN) ?${detail}`);
    console.log('      if npmjs.com already shows the package: wait ~1?min then retry; do not treat as unpublished');
}

/**
 * npm trust list --json 顶层字段：repository / file / environment / permissions
 * （旧形态可能包?claims / workflow_ref 里）
 * @param {any} cfg
 */
function trustFields(cfg) {
    const claims = cfg?.claims ?? {};
    return {
        repo: cfg?.repository ?? claims.repository ?? claims.repo ?? '',
        file: cfg?.file ?? claims.workflow_ref?.file ?? claims.workflowFile ?? claims.file ?? claims.workflow ?? '',
        env: cfg?.environment ?? claims.environment ?? claims.env ?? '',
    };
}

/**
 * @param {any} cfg
 */
function trustMatches(cfg) {
    if (cfg?.raw && typeof cfg.raw === 'string') {
        return cfg.raw.includes(TRUST.repo) && cfg.raw.includes(TRUST.file) && (cfg.raw.includes(TRUST.env) || cfg.raw.includes('NPM_PUBLISH'));
    }
    const { repo, file, env } = trustFields(cfg);
    const perms = new Set(cfg?.permissions ?? []);
    const allowPublish = perms.size === 0 || perms.has('createPackage') || perms.has('publish') || perms.has('npm publish');
    const allowStage = perms.size === 0 || perms.has('createStagedPackage') || perms.has('stage') || perms.has('npm stage publish');
    return repo === TRUST.repo && file === TRUST.file && (env === TRUST.env || env === '') && allowPublish && allowStage;
}

/** @param {any} cfg */
function trustExact(cfg) {
    if (cfg?.raw && typeof cfg.raw === 'string') {
        return trustMatches(cfg) && cfg.raw.includes(TRUST.env);
    }
    const { env } = trustFields(cfg);
    return trustMatches(cfg) && env === TRUST.env;
}

/**
 * @param {any[]} configs
 * @returns {{ matches: boolean, matchKind: string }}
 */
function classifyConfigs(configs) {
    if (configs.find(trustExact)) return { matches: true, matchKind: 'exact' };
    if (configs.find(trustMatches)) return { matches: true, matchKind: 'loose' };
    if (configs.length === 0) return { matches: false, matchKind: 'none' };
    return { matches: false, matchKind: 'mismatch' };
}

/**
 * Live `npm trust list`（不走缓存）.
 * @param {string} name
 */
function listTrustLive(name) {
    const args = ['trust', 'list', name, '--json'];
    const code = currentOtp();
    if (code) args.push(`--otp=${code}`);
    const r = runNpm(args, { silent: true });
    const blob = `${r.stdout}\n${r.stderr}`;
    if (/EOTP|one-time password|auth\/cli/i.test(blob)) {
        return { configs: [], authRequired: true, error: 'EOTP' };
    }
    if (r.status !== 0) {
        return { configs: [], error: blob.slice(0, 200) || `exit ${r.status}` };
    }
    try {
        const data = JSON.parse(r.stdout || '[]');
        if (Array.isArray(data)) return { configs: data };
        if (Array.isArray(data?.configurations)) return { configs: data.configurations };
        if (Array.isArray(data?.items)) return { configs: data.items };
        if (data && typeof data === 'object' && (data.type || data.claims)) {
            return { configs: [data] };
        }
        return { configs: [] };
    } catch {
        if (/github|trusted|repository/i.test(r.stdout)) {
            return { configs: [{ raw: r.stdout }] };
        }
        return { configs: [], error: 'unparseable trust list' };
    }
}

/**
 * @param {CacheFile} cache
 * @param {string} name
 * @param {any[]} configs
 */
function writeTrustCache(cache, name, configs) {
    const { matches, matchKind } = classifyConfigs(configs);
    if (!cache.packages[name]) cache.packages[name] = {};
    cache.packages[name].trust = {
        listedAt: new Date().toISOString(),
        configs,
        matches,
        matchKind,
    };
    saveCache(cache);
    return cache.packages[name].trust;
}

/**
 * @param {CacheFile} cache
 * @param {string} name
 * @param {{ preferLive?: boolean }} [opts]
 * @returns {{ source: 'live'|'cache'|'none', trust?: TrustCacheEntry, authRequired?: boolean, error?: string }}
 */
function resolveTrust(cache, name, opts = {}) {
    const cached = cache.packages[name]?.trust;
    if (cached?.configs && Array.isArray(cached.configs)) {
        const recl = classifyConfigs(cached.configs);
        cached.matches = recl.matches;
        cached.matchKind = recl.matchKind;
    }
    const hasOtp = Boolean(totpSecretRaw || otpStatic);
    const wantLive = opts.preferLive || refresh || hasOtp;

    if (wantLive) {
        if (!hasOtp && refresh) {
            return {
                source: cached ? 'cache' : 'none',
                trust: cached,
                error: '--refresh needs NPM_TOTP_SECRET or --otp to re-list',
            };
        }
        if (!hasOtp) {
            return { source: cached ? 'cache' : 'none', trust: cached };
        }
        const live = listTrustLive(name);
        if (live.authRequired) {
            return { source: cached ? 'cache' : 'none', trust: cached, authRequired: true };
        }
        if (live.error) {
            return { source: cached ? 'cache' : 'none', trust: cached, error: live.error };
        }
        const entry = writeTrustCache(cache, name, live.configs);
        return { source: 'live', trust: entry };
    }

    if (cached) return { source: 'cache', trust: cached };
    return { source: 'none' };
}

function checkStatus() {
    const cache = loadCache();
    console.log('placeholder: status\n');
    console.log(`  Trusted Publisher expect: ${TRUST.repo}  ${TRUST.file}  env=${TRUST.env}  [publish + stage]`);
    console.log(`  cache: ${CACHE_PATH}`);
    if (!totpSecretRaw && !otpStatic) {
        console.log('  tip: put NPM_TOTP_SECRET=<base32> in .env.placeholder.local (gitignored), then pnpm placeholder:trust\n');
    } else {
        console.log('  mode: live trust list + write cache\n');
    }

    let missingPkg = 0;
    let badTrust = 0;
    let unknownTrust = 0;
    let ok = 0;

    for (const spec of STUBS) {
        const resolvedVer = resolvePublishedVersion(cache, spec.name);
        if (!cache.packages[spec.name]) cache.packages[spec.name] = {};
        const ver = resolvedVer.version;

        if (!ver) {
            warnLiveViewMiss(spec.name, 'no live version and no local cache');
            console.log(`  ?${spec.name}  package unknown (cannot confirm published)`);
            missingPkg += 1;
            continue;
        }
        if (resolvedVer.liveMiss) {
            warnLiveViewMiss(spec.name, `using cache ${ver} @ ${cache.packages[spec.name].versionAt ?? '?'}`);
        }
        const verMark = ver === VERSION ? VERSION : `${ver} (!= ${VERSION})`;
        const verSrc = resolvedVer.source === 'cache' ? 'cache; live miss' : resolvedVer.source;
        const resolved = resolveTrust(cache, spec.name, {
            preferLive: Boolean(totpSecretRaw || otpStatic),
        });

        if (resolved.authRequired && !resolved.trust) {
            console.log(`  ? ${spec.name}@${verMark}  trust unknown (EOTP; set NPM_TOTP_SECRET) [${verSrc}]`);
            unknownTrust += 1;
            continue;
        }
        if (!resolved.trust) {
            console.log(`  ? ${spec.name}@${verMark}  trust unknown (no cache; run trust with NPM_TOTP_SECRET) [${verSrc}]`);
            unknownTrust += 1;
            continue;
        }

        const src = resolved.source === 'cache' ? 'cache' : 'live';
        const t = resolved.trust;
        if (t.matches) {
            console.log(`  ?${spec.name}@${verMark}  trust ok [trust=${src}; pkg=${verSrc}]`);
            ok += 1;
        } else if (t.matchKind === 'none') {
            console.log(`  ~ ${spec.name}@${verMark}  trust missing [trust=${src}; pkg=${verSrc}]`);
            badTrust += 1;
        } else {
            console.log(`  ~ ${spec.name}@${verMark}  trust mismatch [trust=${src}; pkg=${verSrc}]`);
            badTrust += 1;
        }
    }

    saveCache(cache);
    console.log(`\nplaceholder: ${ok} ok, ${missingPkg} package missing, ${badTrust} trust missing/mismatch, ${unknownTrust} trust unknown`);
    console.log('  next: pnpm placeholder:publish   or   put NPM_TOTP_SECRET in .env.placeholder.local && pnpm placeholder:trust');
    process.exit(missingPkg + badTrust > 0 ? 1 : 0);
}

/**
 * @param {StubSpec} spec
 * @param {string} dir
 * @param {string | undefined} authToken
 */
function writeStub(spec, dir, authToken) {
    fs.mkdirSync(dir, { recursive: true });
    const pkg = {
        name: spec.name,
        version: VERSION,
        description: spec.description ?? 'SXO placeholder — not for production use.',
        license: 'MIT',
        private: false,
        files: ['README.md'],
    };
    if (spec.os) pkg.os = spec.os;
    if (spec.cpu) pkg.cpu = spec.cpu;
    fs.writeFileSync(path.join(dir, 'package.json'), `${JSON.stringify(pkg, null, 2)}\n`);
    fs.writeFileSync(path.join(dir, 'README.md'), `# ${spec.name}\n\nPlaceholder package (${VERSION}). Reserved for the SXO project.\n`);
    if (authToken) {
        fs.writeFileSync(path.join(dir, '.npmrc'), `//registry.npmjs.org/:_authToken=${authToken}\n`, {
            mode: 0o600,
        });
    }
}

function cmdPublish() {
    const cache = loadCache();
    if (!token) {
        console.warn('placeholder publish: no --token; interactive login may ask OTP (2FA). Tip: --token <npm_access_token> or --otp <code>');
    }

    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'sxo-npm-stubs-'));
    console.log(`placeholder publish: stubs ?${root}`);
    console.log(dryRun ? 'placeholder publish: DRY-RUN' : 'placeholder publish: LIVE');
    console.log(`  cache: ${CACHE_PATH}`);
    console.log('  rule: cache version===0.0.0 ?skip; --refresh forces live npm view. Partial retry OK.\n');

    let published = 0;
    let skipped = 0;

    for (const spec of STUBS) {
        if (!cache.packages[spec.name]) cache.packages[spec.name] = {};
        const cachedVer = cache.packages[spec.name].version;

        if (!refresh && !dryRun && cachedVer === VERSION) {
            console.log(`  ?${spec.name}@${VERSION}  already published ?skip [cache ${cache.packages[spec.name].versionAt ?? ''}]`);
            skipped += 1;
            continue;
        }

        const existing = viewVersion(spec.name);
        if (existing) {
            cache.packages[spec.name].version = existing;
            cache.packages[spec.name].versionAt = new Date().toISOString();
            saveCache(cache);
        }

        if (existing === VERSION && !dryRun) {
            console.log(`  ?${spec.name}@${VERSION}  already published ?skip [live]`);
            skipped += 1;
            continue;
        }

        // Live view briefly 404s but cache already has target version — treat as published.
        if (!existing && cachedVer === VERSION && !dryRun && !refresh) {
            warnLiveViewMiss(spec.name, `skip publish using cache ${VERSION}`);
            console.log(`  ?${spec.name}@${VERSION}  already published ?skip [cache; live miss]`);
            skipped += 1;
            continue;
        }

        if (!existing && !cachedVer) {
            // fall through to publish
        } else if (!existing && cachedVer && cachedVer !== VERSION) {
            warnLiveViewMiss(spec.name, `cache has ${cachedVer}; live unknown ?will attempt publish`);
        }

        const safe = spec.name.replace(/^@/, '').replace(/\//g, '-');
        const dir = path.join(root, safe);
        writeStub(spec, dir, token);

        const args = ['publish'];
        if (spec.name.startsWith('@')) args.push('--access', 'public');
        if (dryRun) args.push('--dry-run');
        const code = currentOtp();
        if (code) args.push(`--otp=${code}`);

        console.log(`\n=== ${spec.name}  npm ${args.join(' ')} ===`);
        const r = runNpm(args, { cwd: dir, silent: true });
        if (r.stdout) process.stdout.write(r.stdout + (r.stdout.endsWith('\n') ? '' : '\n'));
        if (r.stderr) process.stderr.write(r.stderr + (r.stderr.endsWith('\n') ? '' : '\n'));

        if (r.status === 0) {
            published += 1;
            cache.packages[spec.name].version = VERSION;
            cache.packages[spec.name].versionAt = new Date().toISOString();
            saveCache(cache);
            console.log(`  ?${spec.name}@${VERSION}  published + cached`);
            continue;
        }

        const blob = `${r.stdout}\n${r.stderr}`;
        if (/previously published|cannot publish over/i.test(blob)) {
            console.log(`  ?${spec.name}@${VERSION}  already on registry ?skip + cache`);
            skipped += 1;
            cache.packages[spec.name].version = VERSION;
            cache.packages[spec.name].versionAt = new Date().toISOString();
            saveCache(cache);
            continue;
        }

        saveCache(cache);
        fail(`publish failed: ${spec.name} (cache saved ?re-run continues from cached 0.0.0 skips)`);
    }

    saveCache(cache);
    console.log(`\nplaceholder publish: done (published ${published}, skipped ${skipped}).  pnpm placeholder`);
}

function cmdTrust() {
    const cache = loadCache();
    console.log('placeholder trust: Trusted Publisher');
    console.log(`  repo=${TRUST.repo}`);
    console.log(`  file=${TRUST.file}`);
    console.log(`  env=${TRUST.env}`);
    console.log('  permissions: npm publish + npm stage publish');
    console.log(`  cache: ${CACHE_PATH}`);
    console.log(
        `  secrets: totp=${totpSecretRaw ? 'yes' : 'no'} otp6=${otpStatic ? 'yes' : 'no'} token=${token ? 'yes' : 'no'} (flag/env/.env.placeholder.local)`,
    );
    console.log('  rule: skip only when trust list (or cache from a prior list) says already matches\n');

    let configured = 0;
    let skipped = 0;
    let listedThisRun = 0;

    for (const spec of STUBS) {
        if (!cache.packages[spec.name]) cache.packages[spec.name] = {};

        // Prefer trust cache: on hit, skip immediately (no npm view / sleep).
        const cached = !refresh ? cache.packages[spec.name]?.trust : undefined;
        if (cached?.configs && Array.isArray(cached.configs)) {
            const recl = classifyConfigs(cached.configs);
            cached.matches = recl.matches;
            cached.matchKind = recl.matchKind;
            cache.packages[spec.name].trust = cached;
        }
        if (cached?.matches) {
            console.log(`  ?${spec.name}  already configured ?skip [cache ${cached.listedAt}]`);
            skipped += 1;
            continue;
        }

        const resolvedVer = resolvePublishedVersion(cache, spec.name);
        const ver = resolvedVer.version;

        if (!ver) {
            warnLiveViewMiss(spec.name, 'no live version and no local cache ?skip trust');
            continue;
        }
        if (resolvedVer.liveMiss) {
            warnLiveViewMiss(spec.name, `proceeding with cache ${ver}; will still try trust list`);
        }

        // Need a live list (OTP window). Cache miss / non-match / --refresh.
        // Need a live list. Cache miss / non-match / --refresh.
        if (!totpSecretRaw && !otpStatic) {
            saveCache(cache);
            fail(
                `need NPM_TOTP_SECRET (or 6-digit NPM_OTP) to list ${spec.name}. Put secret in .env.placeholder.local then: pnpm placeholder:trust`,
            );
        }

        const live = listTrustLive(spec.name);
        if (live.authRequired) {
            saveCache(cache);
            fail(`EOTP while listing ${spec.name} after ${listedThisRun} list(s) this run. Cache saved ?get a fresh OTP and continue.`);
        }
        if (live.error) {
            saveCache(cache);
            fail(`trust list failed for ${spec.name}: ${live.error}`);
        }

        listedThisRun += 1;
        const entry = writeTrustCache(cache, spec.name, live.configs);
        console.log(`  list ${spec.name} ?${entry.matchKind} (cached)`);

        if (entry.matches) {
            console.log(`  ?${spec.name}  already configured ?skip [live]`);
            skipped += 1;
            continue;
        }

        if (entry.matchKind === 'mismatch') {
            console.log(`  ! ${spec.name}  has trusted publisher that does not match expect:`);
            console.log(`    ${JSON.stringify(live.configs, null, 2).split('\n').join('\n    ')}`);

            // Same-repo stale workflow (e.g. deleted release-npm.yml) → revoke then recreate.
            const stale = live.configs.filter((cfg) => {
                if (trustExact(cfg) || trustMatches(cfg)) return false;
                const { repo } = trustFields(cfg);
                const type = String(cfg?.type ?? 'github').toLowerCase();
                return type === 'github' && repo === TRUST.repo && cfg?.id;
            });
            const foreign = live.configs.filter((cfg) => {
                const { repo } = trustFields(cfg);
                return repo && repo !== TRUST.repo;
            });
            if (foreign.length) {
                fail(`${spec.name}: refuse to touch foreign trust (${foreign.map((c) => trustFields(c).repo).join(', ')}). Revoke manually.`);
            }
            if (!stale.length) {
                fail(`${spec.name}: mismatch with no revocable same-repo ids. Fix manually, then --refresh.`);
            }

            for (const cfg of stale) {
                const revCode = currentOtp();
                if (!revCode) fail('TOTP/OTP missing at revoke time');
                const revArgs = ['trust', 'revoke', spec.name, `--id=${cfg.id}`, `--otp=${revCode}`];
                console.log(`\n=== ${spec.name}  npm trust revoke --id=${cfg.id} (stale ${trustFields(cfg).file || '?'}) ===`);
                if (dryRun) {
                    console.log(`  dry-run would: npm ${revArgs.join(' ')}`);
                    continue;
                }
                const rr = runNpm(revArgs, { silent: true });
                if (rr.stdout) process.stdout.write(rr.stdout + (rr.stdout.endsWith('\n') ? '' : '\n'));
                if (rr.stderr) process.stderr.write(rr.stderr + (rr.stderr.endsWith('\n') ? '' : '\n'));
                if (rr.status !== 0) {
                    saveCache(cache);
                    fail(`trust revoke failed: ${spec.name} id=${cfg.id}`);
                }
                console.log(`  revoked ${cfg.id}`);
                sleep(1000);
            }

            if (dryRun) continue;
            // Fall through to create after revoke.
        }

        // matchKind === 'none' (or revoked mismatch) → create
        const code = currentOtp();
        if (!code) fail('TOTP/OTP missing at create time');
        const args = [
            'trust',
            'github',
            spec.name,
            `--file=${TRUST.file}`,
            `--repo=${TRUST.repo}`,
            `--env=${TRUST.env}`,
            '--allow-publish',
            '--allow-stage-publish',
            '--yes',
            `--otp=${code}`,
        ];
        if (dryRun) {
            console.log(`  dry-run would: npm ${args.join(' ')}`);
            continue;
        }

        console.log(`\n=== ${spec.name}  npm trust github ===`);
        const r = runNpm(args, { silent: true });
        if (r.stdout) process.stdout.write(r.stdout + (r.stdout.endsWith('\n') ? '' : '\n'));
        if (r.stderr) process.stderr.write(r.stderr + (r.stderr.endsWith('\n') ? '' : '\n'));

        if (r.status !== 0) {
            saveCache(cache);
            fail(`trust create failed: ${spec.name} (cache kept; fix and retry — do not assume E409 means ok)`);
        }

        // Confirm via list and cache.
        const again = listTrustLive(spec.name);
        if (!again.authRequired && !again.error) {
            writeTrustCache(cache, spec.name, again.configs);
        } else {
            // Create succeeded; record expected match without second list if OTP died.
            writeTrustCache(cache, spec.name, [
                {
                    type: 'github',
                    claims: {
                        repository: TRUST.repo,
                        workflow_ref: { file: TRUST.file },
                        environment: TRUST.env,
                    },
                    permissions: ['createPackage', 'createStagedPackage'],
                },
            ]);
        }

        configured += 1;
        console.log(`  ok ${spec.name}  configured + cached`);
        sleep(2000);
    }

    saveCache(cache);
    console.log(`\nplaceholder trust: done (configured ${configured}, skipped ${skipped}, listed ${listedThisRun}).`);
}

switch (command) {
    case 'status':
    case 'check':
        checkStatus();
        break;
    case 'publish':
        cmdPublish();
        break;
    case 'trust':
        cmdTrust();
        break;
    default:
        fail(`unknown command \`${command}\`. Use: (default status) | publish | trust`);
}
