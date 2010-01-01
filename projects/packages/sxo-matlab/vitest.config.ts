import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
    resolve: {
        alias: {
            // Resolve harness via workspace package (`projects/tooling/sxo-harness`).
            '@sxo/matlab': path.join(root, 'src/index.ts'),
            '@sxo/core': path.join(root, '../sxo-core/src/index.ts'),
        },
    },
    test: {
        include: ['tests/**/*.test.ts'],
        environment: 'node',
    },
});
