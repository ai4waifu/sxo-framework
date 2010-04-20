/**
 * Isolated Mathematica evaluate worker for crash-containing matrix cases.
 * argv[2] = input. Prints one JSON line: { ok, actual }.
 */
import { Mathematica } from '@sxo/mathematica';

const input = process.argv[2] ?? '';
try {
    const actual = Mathematica.create({ autoSimplify: true }).evaluate(input).toWolfram();
    process.stdout.write(`${JSON.stringify({ ok: true, actual })}\n`);
} catch (e) {
    const actual = e instanceof Error ? e.message : String(e);
    process.stdout.write(`${JSON.stringify({ ok: false, actual })}\n`);
    process.exitCode = 1;
}
