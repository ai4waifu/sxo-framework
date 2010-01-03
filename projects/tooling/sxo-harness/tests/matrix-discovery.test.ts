import { resolveDialectFeatureMatrixEntry } from '@sxo/harness';
import { describe, expect, it } from 'vitest';

describe('feature matrix package discovery', () => {
    it('resolves mathematica entry from sxo.featureMatrix', () => {
        const entry = resolveDialectFeatureMatrixEntry('mathematica');
        expect(entry.replace(/\\/g, '/')).toMatch(/sxo-mathematica\/tests\/feature-matrix\/index\.ts$/);
    });

    it('resolves matlab entry from sxo.featureMatrix', () => {
        const entry = resolveDialectFeatureMatrixEntry('matlab');
        expect(entry.replace(/\\/g, '/')).toMatch(/sxo-matlab\/tests\/feature-matrix\/index\.ts$/);
    });
});
