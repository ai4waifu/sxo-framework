/**
 * Isolated MATLAB evaluate worker for crash-containing matrix cases.
 * argv[2] = input. Prints one JSON line: { ok, actual }.
 */
import { Matlab } from '@sxo/matlab';

const input = process.argv[2] ?? '';
try {
    const actual = Matlab.create({ autoSimplify: true }).evaluate(input).toMatlab();
    process.stdout.write(`${JSON.stringify({ ok: true, actual })}\n`);
} catch (e) {
    const actual = e instanceof Error ? e.message : String(e);
    process.stdout.write(`${JSON.stringify({ ok: false, actual })}\n`);
    process.exitCode = 1;
}
