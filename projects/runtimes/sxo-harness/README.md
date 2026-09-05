# `@sxo/harness`

Private acceptance harness for SXO feature-matrix validation, environment discovery, and reporting.

Not a product runtime. No CLI binary — user commands stay on `@sxo/sxo`.

## Feature Matrix builders

Dialect `feature-matrix` modules should author entries with typed helpers, not raw object literals:

```ts
import { feature, matrix } from '@sxo/harness';

export const featureMatrix = matrix(
  feature('Plus', 'arithmetic')
    .supported()
    .pure()
    .eval('plus.basic', '1 + 2', '3')
    .done(),
  feature('mtimes', 'arithmetic')
    .partial('scalar only')
    .pure()
    .eval('mtimes.scalar', '2 * 3', '6')
    .gap('mtimes.2x2', 'A*B', { expected: 'C', notes: 'symbolic stays Times' })
    .done(),
);
```

Product evaluate / session / N-API paths must not import this package. Only feature-matrix authoring, tests, and `@sxo/sxo` tooling may.

```bash
pnpm --filter @sxo/harness build
pnpm --filter @sxo/harness test
```
