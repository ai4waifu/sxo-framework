/**
 * `wolframscript` — Mathematica Form CLI for `@sxo/mathematica`.
 *
 * Built with `@vmz/commander` (same stack as `@sxo/sxo`). Local evaluate only.
 * WolframScript-shaped aliases (`-code` / `-file`) are peeled into commander
 * subcommands before `parse` — help/catalog/i18n stay on the commander tree.
 */

import { createInterface } from 'node:readline';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assertCatalogCoverage, createCli, loadCatalog, loadLocalesManifest, resolveLocale } from '@vmz/commander';
import { formatDiagnostic, type LocaleCatalog } from '@vmz/diagnostic';
import { Mathematica } from './index.js';
import { installKernelspec, uninstallKernelspec } from './jupyter/install.js';
import { loadNative } from './native.js';

const LOCALES_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', 'locales');
const ENV_KEYS = ['SXO_LOCALE', 'LOCALE', 'LANG', 'LC_ALL'];
const PKG_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function resolveLocalesRoot(): string {
    return LOCALES_ROOT;
}

function activeLocale(argv: string[] = process.argv): string {
    return resolveLocale({
        argv,
        env: process.env,
        manifest: loadLocalesManifest(resolveLocalesRoot()),
        envKeys: ENV_KEYS,
    });
}

function loadDiagCatalog(locale: string): LocaleCatalog {
    return loadCatalog(locale, resolveLocalesRoot());
}

function writeDiag(code: string, detail?: string, locale?: string): void {
    const loc = locale ?? activeLocale();
    const line = formatDiagnostic(
        {
            path: 'wolframscript',
            severity: 'error',
            code,
            args: detail !== undefined ? { detail } : undefined,
        },
        { locale: loc, catalog: loadDiagCatalog(loc) },
    );
    console.error(line);
}

function packageMeta(): { name: string; version: string } {
    const raw = readFileSync(path.join(PKG_ROOT, 'package.json'), 'utf8');
    const pkg = JSON.parse(raw) as { name?: string; version?: string };
    return { name: pkg.name ?? '@sxo/mathematica', version: pkg.version ?? '0.0.0' };
}

function stripShebang(src: string): string {
    if (src.startsWith('#!')) {
        const nl = src.indexOf('\n');
        return nl === -1 ? '' : src.slice(nl + 1);
    }
    return src;
}

function scriptLines(src: string): string[] {
    return stripShebang(src)
        .split(/\r?\n/)
        .map((l) => l.trim())
        .filter((l) => l.length > 0 && !l.startsWith('(*'));
}

function unwrapPrint(line: string): { print: boolean; expr: string } {
    const m = /^Print\s*\[(.*)\]\s*;?\s*$/s.exec(line);
    if (m) return { print: true, expr: m[1]!.trim() };
    const semi = line.endsWith(';') ? line.slice(0, -1).trim() : line;
    return { print: false, expr: semi };
}

function evaluateLine(mma: Mathematica, line: string): { text: string; suppressed: boolean } {
    const { print, expr } = unwrapPrint(line);
    const suppressed = line.trimEnd().endsWith(';') && !print;
    if (!expr) return { text: '', suppressed: true };
    const result = mma.evaluate(expr).toWolfram();
    return { text: result, suppressed: suppressed && !print };
}

type PrintMode = false | 'last' | 'all';

function runFile(mma: Mathematica, file: string, print: PrintMode, scriptMode: boolean): number {
    if (!existsSync(file)) {
        writeDiag('diag.file_not_found', file);
        return 1;
    }
    const lines = scriptLines(readFileSync(file, 'utf8'));
    if (lines.length === 0) return 0;

    let last = '';
    let lastWasPrint = false;
    for (const line of lines) {
        const form = unwrapPrint(line);
        const { text, suppressed } = evaluateLine(mma, line);
        if (form.print && text) {
            process.stdout.write(`${text}\n`);
            last = text;
            lastWasPrint = true;
            continue;
        }
        if (print === 'all' && !suppressed && text) {
            process.stdout.write(`${text}\n`);
        }
        if (!suppressed && text) {
            last = text;
            lastWasPrint = false;
        }
    }

    const shouldPrintLast = print === 'last' || (!scriptMode && print === false);
    if (shouldPrintLast && last && !lastWasPrint) {
        process.stdout.write(`${last}\n`);
    }
    return 0;
}

async function runRepl(mma: Mathematica): Promise<number> {
    const { version } = packageMeta();
    process.stdout.write(`wolframscript ${version} (Mathematica Form — local)\n`);
    process.stdout.write('Type expression, or quit / Exit[]\n\n');

    const rl = createInterface({ input: process.stdin, output: process.stdout });
    let n = 1;
    const prompt = () => {
        rl.setPrompt(`In[${n}]:= `);
        rl.prompt();
    };

    return new Promise((resolve) => {
        prompt();
        rl.on('line', (line) => {
            const trimmed = line.trim();
            if (!trimmed) {
                prompt();
                return;
            }
            if (/^(quit|exit|Exit\[\]:?)$/i.test(trimmed)) {
                rl.close();
                resolve(0);
                return;
            }
            try {
                const out = mma.evaluate(trimmed).toWolfram();
                process.stdout.write(`\nOut[${n}]= ${out}\n\n`);
                n += 1;
            } catch (err) {
                writeDiag('diag.eval_failed', err instanceof Error ? err.message : String(err));
                process.stderr.write('\n');
                n += 1;
            }
            prompt();
        });
        rl.on('close', () => resolve(0));
    });
}

/**
 * Map wolframscript-shaped flags onto commander subcommands.
 * Does not implement a second CLI — only peels aliases before `createCli().parse`.
 */
export function adaptArgv(argv: string[]): string[] {
    const head = argv.slice(0, 2);
    const rest = argv.slice(2);
    if (rest.length === 0) {
        return [...head, 'repl'];
    }

    const out: string[] = [];
    let i = 0;
    while (i < rest.length) {
        const a = rest[i]!;
        if (a === '-c' || a === '-code' || a === '--code') {
            const code = rest[i + 1];
            if (code === undefined) throw new Error('missing_value:code');
            out.push('code', code);
            i += 2;
            continue;
        }
        if (a === '-f' || a === '-file' || a === '--file') {
            const file = rest[i + 1];
            if (file === undefined) throw new Error('missing_value:file');
            out.push('file', file);
            i += 2;
            continue;
        }
        if (a === '-print' || a === '--print') {
            const next = rest[i + 1];
            if (next !== undefined && !next.startsWith('-') && /^all$/i.test(next)) {
                out.push('--print', 'all');
                i += 2;
            } else {
                out.push('--print', 'last');
                i += 1;
            }
            continue;
        }
        if (a === '-version' || a === '--version') {
            out.push('version');
            i += 1;
            continue;
        }
        if (a === '-h' || a === '-help' || a === '--help') {
            out.push('--help');
            i += 1;
            continue;
        }
        if (!a.startsWith('-') && out.length === 0 && /\.(wl|wls|m)$/i.test(a)) {
            out.push('script', a, ...rest.slice(i + 1));
            return [...head, ...out];
        }
        out.push(a);
        i += 1;
    }
    return [...head, ...out];
}

function printModeOf(options: Record<string, string | boolean | string[]>): PrintMode {
    const p = options.print;
    if (p === true || p === 'last') return 'last';
    if (typeof p === 'string' && /^all$/i.test(p)) return 'all';
    return false;
}

function buildCli() {
    const cli = createCli('wolframscript').locales(resolveLocalesRoot(), { envKeys: ENV_KEYS }).intro('cli.intro');

    cli.command('version', 'cli.cmd.version').action((options) => {
        const pkg = packageMeta();
        let engine: string;
        try {
            engine = Mathematica.create().version();
        } catch (err) {
            writeDiag('diag.native_unavailable', err instanceof Error ? err.message : String(err));
            return 1;
        }
        if (options.json === true) {
            console.log(JSON.stringify({ cli: pkg, engine: { version: engine } }));
        } else {
            console.log(`wolframscript ${pkg.version} (${pkg.name})  engine=${engine}`);
        }
        return 0;
    });

    cli.command('code|c', 'cli.cmd.code')
        .option('--json', 'cli.opt.json')
        .action((options) => {
            const expr = options._[0];
            if (!expr) {
                writeDiag('diag.missing_code');
                return 2;
            }
            try {
                const result = Mathematica.create().evaluate(expr).toWolfram();
                if (options.json === true) {
                    console.log(JSON.stringify({ result }));
                } else {
                    console.log(result);
                }
                return 0;
            } catch (err) {
                writeDiag('diag.eval_failed', err instanceof Error ? err.message : String(err));
                return 1;
            }
        });

    cli.command('file|f', 'cli.cmd.file')
        .option('--print [mode]', 'cli.opt.print')
        .option('--json', 'cli.opt.json')
        .action((options) => {
            const file = options._[0];
            if (!file) {
                writeDiag('diag.missing_file');
                return 2;
            }
            try {
                const mode = printModeOf(options) || 'last';
                return runFile(Mathematica.create(), file, mode, false);
            } catch (err) {
                writeDiag('diag.eval_failed', err instanceof Error ? err.message : String(err));
                return 1;
            }
        });

    cli.command('script', 'cli.cmd.script')
        .option('--print [mode]', 'cli.opt.print')
        .passthrough()
        .action((options) => {
            const file = options._[0];
            if (!file) {
                writeDiag('diag.missing_file');
                return 2;
            }
            try {
                return runFile(Mathematica.create(), file, printModeOf(options), true);
            } catch (err) {
                writeDiag('diag.eval_failed', err instanceof Error ? err.message : String(err));
                return 1;
            }
        });

    cli.command('repl', 'cli.cmd.repl').action(async () => {
        try {
            return await runRepl(Mathematica.create());
        } catch (err) {
            writeDiag('diag.native_unavailable', err instanceof Error ? err.message : String(err));
            return 1;
        }
    });

    const jupyter = cli.command('jupyter', 'cli.cmd.jupyter');

    jupyter
        .command('install', 'cli.cmd.jupyter.install')
        .option('--prefix <dir>', 'cli.opt.prefix')
        .option('--name <id>', 'cli.opt.name')
        .action((options) => {
            try {
                const dir = installKernelspec({
                    prefix: typeof options.prefix === 'string' ? options.prefix : undefined,
                    name: typeof options.name === 'string' ? options.name : undefined,
                });
                if (options.json === true) {
                    console.log(JSON.stringify({ installed: dir }));
                } else {
                    console.log(`Installed Jupyter kernelspec at ${dir}`);
                }
                return 0;
            } catch (err) {
                writeDiag('diag.jupyter_install_failed', err instanceof Error ? err.message : String(err));
                return 1;
            }
        });

    jupyter
        .command('uninstall', 'cli.cmd.jupyter.uninstall')
        .option('--prefix <dir>', 'cli.opt.prefix')
        .option('--name <id>', 'cli.opt.name')
        .action((options) => {
            try {
                const removed = uninstallKernelspec({
                    prefix: typeof options.prefix === 'string' ? options.prefix : undefined,
                    name: typeof options.name === 'string' ? options.name : undefined,
                });
                if (options.json === true) {
                    console.log(JSON.stringify({ removed }));
                } else {
                    console.log(removed ? 'Uninstalled Jupyter kernelspec' : 'Kernelspec not found');
                }
                return 0;
            } catch (err) {
                writeDiag('diag.jupyter_install_failed', err instanceof Error ? err.message : String(err));
                return 1;
            }
        });

    jupyter.command('kernel', 'cli.cmd.jupyter.kernel').action((options) => {
        const file = options._[0];
        if (!file) {
            writeDiag('diag.jupyter_missing_connection');
            return 2;
        }
        try {
            loadNative().runJupyterKernel(file);
            return 0;
        } catch (err) {
            writeDiag('diag.jupyter_kernel_failed', err instanceof Error ? err.message : String(err));
            return 1;
        }
    });

    cli.option('--json', 'cli.opt.json');

    assertCatalogCoverage(cli, loadCatalog('en-US', resolveLocalesRoot()));
    return cli;
}

/**
 * CLI entry used by `bin/wolframscript.mjs`.
 */
export async function main(argv: string[] = process.argv): Promise<number> {
    let adapted: string[];
    try {
        adapted = adaptArgv(argv);
    } catch (err) {
        writeDiag('diag.bad_argv', err instanceof Error ? err.message : String(err), activeLocale(argv));
        return 2;
    }
    return buildCli().parse(adapted);
}

export { main as run };
