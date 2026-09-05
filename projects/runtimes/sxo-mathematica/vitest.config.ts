import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
    resolve: {
        alias: {
            '@sxo/harness': path.join(root, '../sxo-harness/src/index.ts'),
            '@sxo/mathematica/jupyter': path.join(root, 'src/jupyter/install.ts'),
            '@sxo/mathematica/cli': path.join(root, 'src/cli.ts'),
            '@sxo/mathematica': path.join(root, 'src/index.ts'),
        },
    },
    test: {
        include: ['tests/**/*.ts'],
        environment: 'node',
    },
});
