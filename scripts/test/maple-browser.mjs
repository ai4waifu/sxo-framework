import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';

const artifactRoot = resolve(process.argv[2] ?? 'projects/platforms/wasm/sxo-lite-unknown-wasm32/lib');
const html = `<!doctype html><meta charset="utf-8"><title>Maple browser WASM verification</title>
<pre id="result">Running Maple browser WASM checks</pre><script type="module">
import init, { Expression, evaluate, simplify } from '/sxo_lite.js';
const checks = [];
function check(condition, label) { if (!condition) throw new Error(label); checks.push(label); }
function rejects(action, label) { let rejected = false; try { action(); } catch { rejected = true; } check(rejected, label); }
try {
    await init();
    const parsed = new Expression('1 / 3', 'maple');
    for (const result of [parsed.evaluate(), evaluate('1 / 3', 'maple')]) {
        check(result.dialect === 'maple', 'explicit Maple dialect');
        check(result.status === 'Exact' && result.coverage === 'Full', 'exact full result');
        check(result.toString() === '1/3', 'exact rational display');
    }
    check(parsed.toString() === '(1 / 3)', 'held Form survives evaluation');
    const sequence = new Expression('1 / 3; 7', 'maple').evaluate();
    check(sequence.status === 'Exact' && sequence.toString() === '7', 'sequence last result');
    rejects(() => new Expression('1 +', 'maple'), 'syntax rejects partial Form');
    rejects(() => new Expression('7', 'unknown'), 'unknown dialect rejected');
    const scalar = new Expression('7', 'maple');
    rejects(() => scalar.toWolfram(), 'no implicit Wolfram conversion');
    rejects(() => scalar.toMatlab(), 'no implicit MATLAB conversion');
    rejects(() => scalar.d('x'), 'derivative unsupported');
    rejects(() => scalar.simplify(), 'handle simplify unsupported');
    rejects(() => scalar.evaluate('simplify'), 'strategy cannot bypass unsupported');
    rejects(() => evaluate('7', 'maple', 'simplify'), 'top-level strategy cannot bypass unsupported');
    rejects(() => simplify('7', 'maple'), 'top-level simplify unsupported');
    check(scalar.isEqual(new Expression('7', 'maple')), 'Maple Form equality');
    document.querySelector('#result').textContent = JSON.stringify({ status: 'passed', checks, userAgent: navigator.userAgent }, null, 2);
} catch (error) {
    document.querySelector('#result').textContent = JSON.stringify({ status: 'failed', error: String(error), checks }, null, 2);
}
</script>`;

await readFile(join(artifactRoot, 'sxo_lite.js'));
await readFile(join(artifactRoot, 'sxo_lite_bg.wasm'));
const server = createServer(async (request, response) => {
    try {
        if (request.url === '/') {
            response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
            response.end(html);
            return;
        }
        const fileName = request.url === '/sxo_lite.js' ? 'sxo_lite.js' : request.url === '/sxo_lite_bg.wasm' ? 'sxo_lite_bg.wasm' : null;
        if (!fileName) {
            response.writeHead(404);
            response.end();
            return;
        }
        response.writeHead(200, { 'Content-Type': fileName.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
        response.end(await readFile(join(artifactRoot, fileName)));
    } catch (error) {
        response.writeHead(500);
        response.end(String(error));
    }
});
server.listen(0, '127.0.0.1', () => {
    const address = server.address();
    if (address && typeof address !== 'string') {
        console.log(`Maple browser verification: http://127.0.0.1:${address.port}`);
    }
});
