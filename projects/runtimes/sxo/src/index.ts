import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { version as engineVersion } from '@sxo/core';
import { assertCatalogCoverage, createCli, loadCatalog, loadLocalesManifest, resolveLocale } from '@vmz/commander';
import { formatDiagnostic, type LocaleCatalog } from '@vmz/diagnostic';

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
            path: 'sxo',
            severity: 'error',
            code,
            args: detail !== undefined ? { detail } : undefined,
        },
        { locale: loc, catalog: loadDiagCatalog(loc) },
    );
    console.error(line);
}

function cliPackageMeta(): { name: string; version: string } {
    const raw = readFileSync(path.join(PKG_ROOT, 'package.json'), 'utf8');
    const pkg = JSON.parse(raw) as { name?: string; version?: string };
    return { name: pkg.name ?? '@sxo/sxo', version: pkg.version ?? '0.0.0' };
}

function buildCli() {
    const cli = createCli('sxo').locales(resolveLocalesRoot(), { envKeys: ENV_KEYS }).intro('cli.intro').option('--json', 'cli.opt.json');

    cli.command('version', 'cli.cmd.version').action((options) => {
        const pkg = cliPackageMeta();
        let core: string;
        try {
            core = engineVersion();
        } catch (err) {
            writeDiag('diag.native_unavailable', err instanceof Error ? err.message : String(err));
            return 1;
        }
        const body = {
            cli: { name: pkg.name, version: pkg.version },
            engine: { version: core },
        };
        if (options.json === true) {
            console.log(JSON.stringify(body));
        } else {
            console.log(`${pkg.name}@${pkg.version}  engine=${core}`);
        }
        return 0;
    });

    cli.command('doctor', 'cli.cmd.doctor').action((options) => {
        const pkg = cliPackageMeta();
        let engine: string | null = null;
        let ok = true;
        let error: string | undefined;
        try {
            engine = engineVersion();
        } catch (err) {
            ok = false;
            error = err instanceof Error ? err.message : String(err);
        }
        const body = {
            ok,
            cli: { name: pkg.name, version: pkg.version },
            engine: engine ? { version: engine } : null,
            native: { ok, error: error ?? null },
            node: process.version,
            platform: `${process.platform}-${process.arch}`,
        };
        if (options.json === true) {
            console.log(JSON.stringify(body));
        } else if (ok) {
            console.log(
                [
                    `ok        yes`,
                    `cli       ${pkg.name}@${pkg.version}`,
                    `engine    ${engine}`,
                    `node      ${process.version}`,
                    `platform  ${body.platform}`,
                ].join('\n'),
            );
        } else {
            writeDiag('diag.native_unavailable', error ?? 'unknown');
            if (!options.json) {
                console.log([`ok        no`, `cli       ${pkg.name}@${pkg.version}`, `node      ${process.version}`].join('\n'));
            }
        }
        return ok ? 0 : 1;
    });

    assertCatalogCoverage(cli, loadCatalog('en-US', resolveLocalesRoot()));
    return cli;
}

/**
 * CLI entry used by `bin/sxo.mjs`.
 * Tooling only (version / doctor) — symbolic compute stays in library APIs.
 */
export async function main(argv: string[] = process.argv): Promise<number> {
    return buildCli().parse(argv);
}

export { main as run };
