import { mkdtemp, readFile, writeFile, access, mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const workspaceRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const oakRoot = resolve(process.argv[2] ?? join(workspaceRoot, '..', 'oaks'));
const dialectSource = join(workspaceRoot, 'projects/dialects/sxo-dialect-maple/src');
const manifest = await readFile(join(workspaceRoot, 'Cargo.toml'), 'utf8');
const athenaDependency = manifest.match(/^athena = \{[^\r\n]+\}$/m)?.[0];
if (!athenaDependency || !athenaDependency.includes('rev =')) {
    throw new Error("Expected the product's pinned Athena dependency");
}
for (const path of [
    join(oakRoot, 'projects/oak-core/Cargo.toml'),
    join(oakRoot, 'examples/oak-maple/Cargo.toml'),
    join(dialectSource, 'lib.rs'),
    join(dialectSource, 'parse.rs'),
]) {
    await access(path);
}
const validationRoot = await mkdtemp(join(tmpdir(), 'sxo-maple-source-'));
const cargoPath = (path) => JSON.stringify(path.replaceAll('\\', '/'));
await mkdir(join(validationRoot, 'src'));
await writeFile(
    join(validationRoot, 'Cargo.toml'),
    [
        '[package]',
        'name = "sxo-maple-source-validation"',
        'version = "0.0.0"',
        'edition = "2021"',
        'publish = false',
        '',
        '[workspace]',
        '',
        '[dependencies]',
        athenaDependency,
        'oak-core = { path = ' + cargoPath(join(oakRoot, 'projects/oak-core')) + ', default-features = false }',
        'oak-maple = { path = ' + cargoPath(join(oakRoot, 'examples/oak-maple')) + ', default-features = false }',
        '',
    ].join('\n'),
    'utf8',
);
await writeFile(
    join(validationRoot, 'src/lib.rs'),
    [
        '#[path = ' + cargoPath(join(dialectSource, 'lib.rs')) + ']',
        'mod dialect;',
        'pub use dialect::*;',
        '#[path = ' + cargoPath(join(dialectSource, 'parse.rs')) + ']',
        'pub mod parse;',
        '',
    ].join('\n'),
    'utf8',
);
console.log('Development-only Maple source validation. Product git dependencies are unchanged.');
console.log('Validation manifest:', join(validationRoot, 'Cargo.toml'));
const result = spawnSync(
    'cargo',
    ['+nightly', 'test', '--manifest-path', join(validationRoot, 'Cargo.toml'), '--target-dir', join(workspaceRoot, 'target')],
    { cwd: validationRoot, stdio: 'inherit' },
);
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
