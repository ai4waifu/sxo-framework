/**
 * Browser-runtime smoke: load the shipped `.wasm` binary (via wasm-bindgen web target)
 * and assert the same direct vs parse→handle matrix parity locked in `sxo-lite-wasm` Rust tests.
 */
import { Expression, ensureReady, evaluate } from '@sxo/lite-unknown-wasm32';
import { beforeAll, describe, expect, it } from 'vitest';

const MATLAB_CASES: ReadonlyArray<readonly [string, string]> = [
    ['(1+2)*3', '9'],
    ['[1, 2; 3, 4]*[5, 6; 7, 8]', '[19, 22; 43, 50]'],
    ['[1+i, 0; 0, 1-i]*[1, i; -i, 1]', '[1 + i, -1 + i; -1 - i, 1 - i]'],
    ['[1+i, 2; 3, 4].*[1, i; 0, 1]', '[1 + i, 2*i; 0, 4]'],
    ['triu([1+i, 2; 3, 4-i])', '[1 + i, 2; 0, 4 - i]'],
    ['sum([1+i, 2; 3, 4-i], 1)', '[4 + i, 6 - i]'],
    ['0/0', 'NaN'],
];

const MMA_CASES: ReadonlyArray<readonly [string, string]> = [
    ['Dot[{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}]', '{{19, 22}, {43, 50}}'],
    ['{{1 + I, 2}, {3, 4}}*{{1, I}, {0, 1}}', '{{1 + I, 2*I}, {0, 4}}'],
    ['Riffle[{1, 2}, {I, 2 I}]', '{1, I, 2, 2*I}'],
    ['0/0', 'Indeterminate'],
];

function assertWasmParity(dialect: 'matlab' | 'mathematica', input: string, expected: string): void {
    const render = (expr: Expression) => (dialect === 'matlab' ? expr.toMatlab() : expr.toWolfram());

    const direct = evaluate(input, dialect, 'none');
    const directText = render(direct);

    const parsed = new Expression(input, dialect);
    const viaHandle = parsed.evaluate('none');
    const handleText = render(viaHandle);

    expect(directText, `direct vs handle for ${input}`).toBe(handleText);
    expect(directText, `expected for ${input}`).toBe(expected);
    expect(direct.status, `status parity for ${input}`).toBe(viaHandle.status);
    expect(direct.coverage, `coverage parity for ${input}`).toBe(viaHandle.coverage);
    expect(directText, `Null guard for ${input}`).not.toBe('Null');
}

describe('@sxo/lite wasm session matrix parity', () => {
    beforeAll(async () => {
        await ensureReady();
    });

    it('loads a semver version from the wasm module', () => {
        const v = evaluate('1+1', 'matlab', 'none').toMatlab();
        expect(v).toBe('2');
    });

    it.each(MATLAB_CASES)('matlab %s', (input, expected) => {
        assertWasmParity('matlab', input, expected);
    });

    it.each(MMA_CASES)('mathematica %s', (input, expected) => {
        assertWasmParity('mathematica', input, expected);
    });
});
