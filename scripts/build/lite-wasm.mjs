/**
 * Build sxo-lite-wasm → `@sxo/lite-unknown-wasm32`:
 *   lib/  — wasm-bindgen glue + `.wasm` (generated)
 *   src/  — TypeScript rebind (hand-written)
 *   dist/ — tsc output of src/
 */
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const crate = path.join(root, 'projects/compilers/sxo-lite-wasm');
const outPkg = path.join(root, 'projects/runtimes/sxo-lite-unknown-wasm32');
const libDir = path.join(outPkg, 'lib');
const distDir = path.join(outPkg, 'dist');
const staging = path.join(crate, '.wasm-pack-out');

function run(cmd, args, cwd) {
    const r = spawnSync(cmd, args, { cwd, stdio: 'inherit', shell: false });
    if (r.error) {
        console.error(r.error);
        process.exit(1);
    }
    if (r.status !== 0) process.exit(r.status ?? 1);
}

if (existsSync(staging)) rmSync(staging, { recursive: true, force: true });
mkdirSync(staging, { recursive: true });

if (existsSync(libDir)) rmSync(libDir, { recursive: true, force: true });
mkdirSync(libDir, { recursive: true });

run('wasm-pack', ['build', '--target', 'web', '--release', '--out-dir', '.wasm-pack-out', '--out-name', 'sxo_lite'], crate);

for (const name of readdirSync(staging)) {
    if (name === 'package.json' || name === 'README.md' || name === '.gitignore') continue;
    copyFileSync(path.join(staging, name), path.join(libDir, name));
}

rmSync(staging, { recursive: true, force: true });

const staleNested = path.join(crate, 'projects');
if (existsSync(staleNested)) rmSync(staleNested, { recursive: true, force: true });

const gluePkg = path.join(libDir, 'package.json');
if (!existsSync(gluePkg)) {
    writeFileSync(gluePkg, `${JSON.stringify({ type: 'module' }, null, 2)}\n`);
}

console.log(`lite-wasm lib ← ${libDir}`);
for (const name of readdirSync(libDir)) console.log(`  lib/${name}`);

if (existsSync(distDir)) rmSync(distDir, { recursive: true, force: true });
run(process.execPath, [path.join(root, 'node_modules/typescript/bin/tsc'), '-p', 'tsconfig.json'], outPkg);

console.log(`lite-wasm dist ← ${distDir}`);
if (existsSync(distDir)) {
    for (const name of readdirSync(distDir)) console.log(`  dist/${name}`);
}
