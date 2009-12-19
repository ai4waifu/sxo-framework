import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
    resolve: {
        alias: {
            '@sxo/simple-math': path.join(root, 'src/index.ts'),
        },
    },
    test: {
        include: ['tests/**/*.ts'],
        environment: 'node',
    },
});
