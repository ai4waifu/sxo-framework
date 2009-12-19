import { describe, expect, it } from 'vitest';
import {
    entry,
    evalCase,
    feature,
    gapCase,
    matrix,
    negativeCase,
    parseCase,
    plotCase,
    roundtripCase,
    validateFeatureMatrix,
} from '../src/index.js';

describe('feature matrix builders', () => {
    it('builds a supported entry with fluent eval cases', () => {
        const plus = feature('Plus', 'arithmetic').supported().pure().eval('plus.basic', '1+2', '3').eval('plus.nary', 'Plus[1,2]', '3').done();

        expect(plus).toEqual({
            name: 'Plus',
            category: 'arithmetic',
            status: 'supported',
            effect: 'pure',
            cases: [
                { id: 'plus.basic', kind: 'eval', input: '1+2', expected: '3' },
                { id: 'plus.nary', kind: 'eval', input: 'Plus[1,2]', expected: '3' },
            ],
        });
    });

    it('records partial notes and gap options', () => {
        const mtimes = feature('mtimes', 'arithmetic')
            .partial('scalar only')
            .pure()
            .eval('mtimes.scalar', '2*3', '6')
            .gap('mtimes.2x2', '[1,2]*[3,4]', { expected: '[3,8]', notes: 'symbolic stays Times' })
            .done();

        expect(mtimes.status).toBe('partial');
        expect(mtimes.notes).toBe('scalar only');
        expect(mtimes.cases[1]).toEqual({
            id: 'mtimes.2x2',
            kind: 'gap',
            input: '[1,2]*[3,4]',
            expected: '[3,8]',
            notes: 'symbolic stays Times',
        });
    });

    it('covers standalone case constructors', () => {
        expect(evalCase('e', '1', '1')).toEqual({ id: 'e', kind: 'eval', input: '1', expected: '1' });
        expect(parseCase('p', 'x', 'x')).toEqual({ id: 'p', kind: 'parse', input: 'x', expected: 'x' });
        expect(roundtripCase('r', '{1}', '{1}')).toEqual({ id: 'r', kind: 'roundtrip', input: '{1}', expected: '{1}' });
        expect(plotCase('pl', 'Plot[x,{x,0,1}]', { expected: '<svg' })).toEqual({
            id: 'pl',
            kind: 'plot',
            input: 'Plot[x,{x,0,1}]',
            expected: '<svg',
        });
        expect(negativeCase('n', 'bad', { forbidden: 'ok' })).toEqual({
            id: 'n',
            kind: 'negative',
            input: 'bad',
            forbidden: 'ok',
        });
        expect(gapCase('g', 'todo')).toEqual({ id: 'g', kind: 'gap', input: 'todo' });
    });

    it('supports entry() and matrix() assembly', () => {
        const m = matrix(
            entry('Abs', 'arithmetic', 'supported', 'pure', [evalCase('abs.neg', 'Abs[-1]', '1')]),
            feature('Future', 'misc').planned('not started').pure().gap('future.1', 'Future[]').done(),
        );
        expect(m).toHaveLength(2);
        expect(validateFeatureMatrix(m).ok).toBe(true);
    });

    it('throws when status or effect is omitted', () => {
        expect(() => feature('X', 'y').pure().done()).toThrow(/missing status/);
        expect(() => feature('X', 'y').supported().done()).toThrow(/missing effect/);
    });

    it('carries host backend device defaults', () => {
        const e = feature('GpuOp', 'tensor')
            .supported()
            .pure()
            .host('native')
            .backend('internal-titan')
            .device('cuda:0')
            .eval('gpu.1', 'op()', '1', { host: 'wasm' })
            .done();
        expect(e.host).toBe('native');
        expect(e.backend).toBe('internal-titan');
        expect(e.device).toBe('cuda:0');
        expect(e.cases[0]?.host).toBe('wasm');
    });
});
