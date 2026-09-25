#!/usr/bin/env node
/**
 * Reference changelog from git tags (one bullet per commit).
 *
 *   pnpm change-logs --version 0.0.3
 *   pnpm change-logs --from v0.0.2 --to v0.0.3
 *   pnpm change-logs --version 0.0.3 --write
 *   pnpm change-logs --tags
 *
 * Output lines: `- <subject> (@user)` — reference only; hand-edit releases/vX.Y.Z.md for publish.
 * `--write` lands in gitignored `documentation/maintenance/releases/vX.Y.Z.reference.md`.
 */

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const RELEASES_DIR = path.join(ROOT, 'documentation', 'maintenance', 'releases');
const AUTHOR_GITHUB_PATH = path.join(ROOT, 'documentation', 'maintenance', 'author-github.json');
const DEFAULT_REPO = 'ai4waifu/sxo-framework';

/**
 * @typedef {{ id?: number, login?: string }} GithubAuthor
 */

/**
 * @param {unknown} value
 * @returns {GithubAuthor | null}
 */
function parseAuthorEntry(value) {
    if (typeof value === 'string') {
        const login = value.trim();
        return login ? { login } : null;
    }
    if (typeof value === 'number' && Number.isFinite(value)) {
        return { id: value };
    }
    if (value && typeof value === 'object' && !Array.isArray(value)) {
        const raw = value;
        const login = raw.login != null ? String(raw.login).trim() : '';
        const id = raw.id != null ? Number(raw.id) : undefined;
        const hasId = id != null && Number.isFinite(id);
        if (login || hasId) {
            return { id: hasId ? id : undefined, login: login || undefined };
        }
    }
    return null;
}

/**
 * @returns {Record<string, GithubAuthor>}
 */
function loadAuthorGithubMap() {
    try {
        const text = fs.readFileSync(AUTHOR_GITHUB_PATH, 'utf8').replace(/^\uFEFF/, '');
        const raw = JSON.parse(text);
        if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return {};
        const out = {};
        for (const [email, entry] of Object.entries(raw)) {
            const parsed = parseAuthorEntry(entry);
            if (parsed) out[String(email).trim().toLowerCase()] = parsed;
        }
        return out;
    } catch {
        return {};
    }
}

const AUTHOR_GITHUB = loadAuthorGithubMap();

const GITMOJI_RE = /^(\p{Extended_Pictographic}\uFE0F?)\s+/u;

/** @type {Record<string, 'features' | 'fixes' | 'breaking' | 'other'>} */
const GITMOJI_SECTION = {
    '✨': 'features',
    '🎨': 'features',
    '🚀': 'features',
    '🐛': 'fixes',
    '🚑': 'fixes',
    '🔥': 'fixes',
    '💥': 'breaking',
    '♻️': 'other',
    '🔧': 'other',
    '📝': 'other',
    '👷': 'other',
    '🧹': 'other',
    '⬆️': 'other',
    '🧪': 'other',
    '🔨': 'other',
    '📦': 'other',
};

const SECTION_META = {
    features: { heading: '## ✨ Features', empty: '(none)' },
    fixes: { heading: '## 🐛 Bug Fixes', empty: '(none)' },
    breaking: { heading: '## ⚠️ Breaking Changes', empty: '(none)' },
    other: { heading: '## 📝 Other', empty: '(none)' },
};

const argv = process.argv.slice(2);

function fail(msg) {
    console.error(`change-logs: ${msg}`);
    process.exit(1);
}

function takeFlag(args, flag) {
    const i = args.indexOf(flag);
    if (i >= 0 && args[i + 1] && !args[i + 1].startsWith('-')) return args[i + 1];
    const eq = args.find((a) => a.startsWith(`${flag}=`));
    if (eq) return eq.slice(flag.length + 1);
    return undefined;
}

/**
 * @param {string[]} args
 * @param {string} flag
 */
function hasFlag(args, flag) {
    return args.includes(flag) || args.some((a) => a.startsWith(`${flag}=`));
}

/**
 * @param {string[]} gitArgs
 * @returns {string}
 */
function gitOutput(gitArgs) {
    const r = spawnSync('git', gitArgs, {
        cwd: ROOT,
        encoding: 'utf8',
        shell: false,
    });
    if (r.status !== 0) {
        fail(r.stderr?.trim() || `git ${gitArgs.join(' ')} failed`);
    }
    return String(r.stdout ?? '').trimEnd();
}

/**
 * @returns {string[]}
 */
function listVersionTags() {
    const out = gitOutput(['tag', '-l', 'v*', '--sort=version:refname']);
    return out ? out.split('\n').filter(Boolean) : [];
}

/**
 * @param {string} version
 * @returns {string}
 */
function normalizeVersion(version) {
    return version.replace(/^v/, '');
}

/**
 * @param {string} version
 * @returns {string}
 */
function versionTag(version) {
    const v = normalizeVersion(version);
    return `v${v}`;
}

/**
 * @param {string} version
 * @returns {string | undefined}
 */
function previousVersionTag(version) {
    const target = versionTag(version);
    const tags = listVersionTags();
    const idx = tags.indexOf(target);
    if (idx > 0) return tags[idx - 1];
    if (idx === 0) return undefined;
    const v = normalizeVersion(version);
    const parts = v.split('.').map((n) => Number.parseInt(n, 10));
    if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) return undefined;
    if (parts[2] > 0) {
        const prev = `${parts[0]}.${parts[1]}.${parts[2] - 1}`;
        const tag = versionTag(prev);
        return tags.includes(tag) ? tag : undefined;
    }
    return undefined;
}

/**
 * @param {string} subject
 */
function leadingGitmoji(subject) {
    const m = subject.match(GITMOJI_RE);
    return m ? m[1] : null;
}

/**
 * @param {string} subject
 */
function stripGitmoji(subject) {
    return subject.replace(GITMOJI_RE, '').trim();
}

/**
 * @param {string} email
 * @returns {GithubAuthor | null}
 */
function githubFromNoreplyEmail(email) {
    const withId = email.match(/^(\d+)\+([^@+]+)@users\.noreply\.github\.com$/i);
    if (withId) return { id: Number(withId[1]), login: withId[2] };
    const loginOnly = email.match(/^([^@+]+)@users\.noreply\.github\.com$/i);
    if (loginOnly) return { login: loginOnly[1] };
    return null;
}

/**
 * @param {GithubAuthor} author
 */
function contributorKey(author) {
    if (author.id != null) return `id:${author.id}`;
    if (author.login) return `login:${author.login}`;
    return null;
}

/**
 * @param {GithubAuthor} author
 */
function displayLogin(author) {
    if (author.login) return author.login;
    if (author.id != null) return `user-${author.id}`;
    return 'unknown';
}

/**
 * @param {GithubAuthor} author
 */
function profileUrl(author) {
    if (author.login) return `https://github.com/${author.login}`;
    if (author.id != null) return `https://github.com/user/${author.id}`;
    return '#';
}

/**
 * @param {GithubAuthor} author
 */
function avatarUrl(author) {
    if (author.id != null) return `https://avatars.githubusercontent.com/u/${author.id}?s=100`;
    if (author.login) return `https://github.com/${author.login}.png?s=100`;
    return '';
}

/**
 * @param {string} email
 * @returns {GithubAuthor | null}
 */
function resolveGithubAuthor(email) {
    const fromNoreply = githubFromNoreplyEmail(email);
    if (fromNoreply) return fromNoreply;
    const mapped = AUTHOR_GITHUB[email.trim().toLowerCase()];
    return mapped ?? null;
}

/**
 * @param {string} email
 * @param {string} author
 */
function authorMention(email, author) {
    const gh = resolveGithubAuthor(email);
    if (gh) {
        const label = `@${displayLogin(gh)}`;
        return `[${label}](${profileUrl(gh)})`;
    }
    const name = author.trim();
    if (name) return `@${name.replace(/\s+/g, '')}`;
    const local = email.split('@')[0]?.trim();
    return local ? `@${local}` : '@unknown';
}

/**
 * @param {string} fromRef
 * @param {string} toRef
 */
function collectCommits(fromRef, toRef) {
    const range = fromRef ? `${fromRef}..${toRef}` : toRef;
    const out = gitOutput(['log', range, '--no-merges', '--format=%H%x1f%ae%x1f%an%x1f%s']);
    if (!out) return [];
    return out
        .split('\n')
        .filter(Boolean)
        .map((line) => {
            const [hash, email, author, subject] = line.split('\x1f');
            const gitmoji = leadingGitmoji(subject);
            const section = (gitmoji && GITMOJI_SECTION[gitmoji]) || 'other';
            return { hash, email, author, subject, gitmoji, section, body: stripGitmoji(subject) };
        });
}

/**
 * @param {ReturnType<typeof collectCommits>} commits
 * @returns {GithubAuthor[]}
 */
function collectContributors(commits) {
    const byKey = new Map();
    for (const commit of commits) {
        const gh = resolveGithubAuthor(commit.email);
        if (!gh) continue;
        const key = contributorKey(gh);
        if (!key) continue;
        const existing = byKey.get(key);
        if (!existing) {
            byKey.set(key, gh);
            continue;
        }
        byKey.set(key, {
            id: existing.id ?? gh.id,
            login: existing.login ?? gh.login,
        });
    }
    return [...byKey.values()].sort((a, b) => displayLogin(a).localeCompare(displayLogin(b)));
}

/**
 * @param {ReturnType<typeof collectCommits>} commits
 */
function groupCommits(commits) {
    const groups = { features: [], fixes: [], breaking: [], other: [] };
    for (const commit of commits) {
        groups[commit.section].push(commit);
    }
    return groups;
}

/**
 * @param {{ email: string, author: string, body: string }} commit
 */
function commitBullet(commit) {
    return `- ${commit.body} (${authorMention(commit.email, commit.author)})`;
}

/**
 * @param {GithubAuthor[]} contributors
 */
function renderContributorWall(contributors) {
    if (contributors.length === 0) {
        return '(none — no GitHub login from commit email; add `documentation/maintenance/author-github.json` if needed)';
    }
    if (contributors.length > 12) {
        const users = contributors
            .map((c) => c.login)
            .filter(Boolean)
            .join(',');
        if (!users) {
            return '(none — contributors have numeric ids only; use the table layout below 12 people or add `login` to author-github.json)';
        }
        return [
            `<a href="https://github.com/${DEFAULT_REPO}/graphs/contributors">`,
            `  <img src="https://contrib.rocks/image?users=${users}&columns=9" alt="Contributors" />`,
            '</a>',
        ].join('\n');
    }
    const columns = Math.min(contributors.length, 6);
    const rows = [];
    for (let i = 0; i < contributors.length; i += columns) {
        const slice = contributors.slice(i, i + columns);
        const rowCells = slice
            .map((contributor) => {
                const login = displayLogin(contributor);
                const href = profileUrl(contributor);
                const src = avatarUrl(contributor);
                return `<td align="center"><a href="${href}"><img src="${src}" width="64" height="64" alt="@${login}"/><br /><sub><b>${login}</b></sub></a></td>`;
            })
            .join('\n');
        rows.push(`<tr>\n${rowCells}\n</tr>`);
    }
    return ['<table>', '<tbody>', ...rows, '</tbody>', '</table>'].join('\n');
}

/**
 * @param {string} version
 * @param {string | undefined} fromRef
 * @param {string} toRef
 * @param {ReturnType<typeof groupCommits>} groups
 * @param {string} contributors
 * @param {number} commitCount
 */
function renderReference(version, fromRef, toRef, groups, contributors, commitCount) {
    const rangeLabel = fromRef ? `${fromRef}..${toRef}` : toRef;
    const lines = [
        `# Reference: v${version} (\`${rangeLabel}\`)`,
        '',
        '> Commit index for drafting `documentation/maintenance/releases/v' +
            version +
            '.md`. Temporary file — do not commit or publish as the GitHub Release body.',
        '',
    ];
    for (const key of ['features', 'fixes', 'breaking', 'other']) {
        const meta = SECTION_META[key];
        lines.push(meta.heading, '');
        const items = groups[key];
        if (items.length === 0) {
            lines.push(meta.empty, '');
        } else {
            for (const commit of items) {
                lines.push(commitBullet(commit));
            }
            lines.push('');
        }
    }
    lines.push('## 👥 Contributors', '', contributors, '', '---', '', `${commitCount} commit(s) in range.`, '');
    return lines.join('\n');
}

function cmdTags() {
    const tags = listVersionTags();
    if (tags.length === 0) {
        console.log('(no v* tags)');
        return;
    }
    for (const tag of tags) {
        const short = gitOutput(['rev-parse', '--short', tag]);
        console.log(`${tag}\t${short}`);
    }
}

function resolveRange(args) {
    const version = takeFlag(args, '--version');
    const from = takeFlag(args, '--from');
    const to = takeFlag(args, '--to');

    if (version) {
        const tag = versionTag(version);
        gitOutput(['rev-parse', '--verify', `${tag}^{commit}`]);
        const prev = previousVersionTag(version);
        return {
            version: normalizeVersion(version),
            fromRef: prev,
            toRef: tag,
        };
    }

    if (!to) fail('need --version=X.Y.Z or --to=REF (optional --from=REF)');

    const toRef = to.startsWith('v') || /^[0-9a-f]{7,40}$/i.test(to) ? to : versionTag(to);
    gitOutput(['rev-parse', '--verify', `${toRef}^{commit}`]);
    const fromRef = from ?? undefined;
    if (fromRef) gitOutput(['rev-parse', '--verify', `${fromRef}^{commit}`]);

    const versionMatch = /^v?(\d+\.\d+\.\d+)$/.exec(toRef);
    return {
        version: versionMatch ? versionMatch[1] : toRef.replace(/^v/, ''),
        fromRef,
        toRef,
    };
}

function main() {
    if (hasFlag(argv, '--help') || hasFlag(argv, '-h')) {
        process.stdout.write(`change-logs — per-commit reference for release note drafting

Usage:
  pnpm change-logs --version 0.0.3
  pnpm change-logs --from v0.0.2 --to v0.0.3
  pnpm change-logs --version 0.0.3 --write
  pnpm change-logs --tags

Output (stdout): grouped bullets, one per commit:
  - <subject without gitmoji> (@user)

Options:
  --version=X.Y.Z   Range: previous v* tag .. vX.Y.Z
  --from=REF        Explicit range start (exclusive)
  --to=REF          Explicit range end (tip)
  --write           Write documentation/maintenance/releases/vX.Y.Z.reference.md (gitignored)
  --tags            List version tags

Author map: documentation/maintenance/author-github.json
  { "email@example": { "id": 12345, "login": "handle" } }
  Prefer numeric id for stable avatars when login may change.
`);
        return;
    }

    if (hasFlag(argv, '--tags')) {
        cmdTags();
        return;
    }

    const { version, fromRef, toRef } = resolveRange(argv);
    const commits = collectCommits(fromRef, toRef);
    const groups = groupCommits(commits);
    const contributors = collectContributors(commits);
    const contributorWall = renderContributorWall(contributors);
    const notes = renderReference(version, fromRef, toRef, groups, contributorWall, commits.length);

    const rangeLabel = fromRef ? `${fromRef}..${toRef}` : toRef;
    console.error(`change-logs: ${rangeLabel} — ${commits.length} commit(s)`);

    if (hasFlag(argv, '--write')) {
        fs.mkdirSync(RELEASES_DIR, { recursive: true });
        const outPath = path.join(RELEASES_DIR, `v${version}.reference.md`);
        fs.writeFileSync(outPath, notes, 'utf8');
        console.error(`change-logs: wrote ${path.relative(ROOT, outPath)}`);
    }

    process.stdout.write(`${notes}\n`);
}

main();
