import { spawnSync } from 'node:child_process';
import process from 'node:process';

/** Result of an isolated evaluate worker. */
export type IsolatedEvalResult = {
    actual: string;
    threw: boolean;
    timedOut: boolean;
    exitCode: number | null;
    signal: string | null;
    stderr: string;
};

export type IsolatedEvalSpec = {
    /** Absolute path to a Node ESM worker. Receives `input` as argv[2]. */
    worker: string;
    input: string;
    timeoutMs?: number;
    env?: NodeJS.ProcessEnv;
    execPath?: string;
};

/**
 * Run a dialect evaluate worker in a child process.
 *
 * Worker contract: print one JSON line `{ "ok": boolean, "actual": string }` to stdout.
 * Process crash / timeout / non-zero exit without JSON is recorded as threw.
 */
export function runIsolatedEval(spec: IsolatedEvalSpec): IsolatedEvalResult {
    const timeoutMs = spec.timeoutMs ?? 8000;
    const child = spawnSync(spec.execPath ?? process.execPath, [spec.worker, spec.input], {
        encoding: 'utf8',
        timeout: timeoutMs,
        env: spec.env ? { ...process.env, ...spec.env } : process.env,
        windowsHide: true,
    });

    const timedOut =
        (typeof child.error === 'object' && child.error !== null && 'code' in child.error && child.error.code === 'ETIMEDOUT') ||
        child.signal === 'SIGTERM';
    const stderr = child.stderr ?? '';
    const stdout = (child.stdout ?? '').trim();
    const exitCode = child.status;
    const signal = child.signal;

    if (timedOut) {
        return {
            actual: `timed out after ${timeoutMs}ms`,
            threw: true,
            timedOut: true,
            exitCode,
            signal,
            stderr,
        };
    }

    if (stdout) {
        try {
            const line = stdout.split(/\r?\n/).filter(Boolean).at(-1) ?? stdout;
            const parsed = JSON.parse(line) as { ok?: boolean; actual?: string };
            if (typeof parsed.actual === 'string') {
                return {
                    actual: parsed.actual,
                    threw: parsed.ok === false || exitCode !== 0,
                    timedOut: false,
                    exitCode,
                    signal,
                    stderr,
                };
            }
        } catch {
            // fall through
        }
    }

    const crashNote =
        signal != null
            ? `process signal ${signal}`
            : exitCode != null && exitCode !== 0
              ? `exit code ${exitCode}`
              : (child.error?.message ?? 'isolated evaluate failed');
    return {
        actual: crashNote,
        threw: true,
        timedOut: false,
        exitCode,
        signal,
        stderr: stderr || stdout,
    };
}
