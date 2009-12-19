# `@sxo/harness`

Private **R&D capability** package for SXO: feature-matrix builders / validation / case runners / reporters, plus (planned) environment discovery and benchmark orchestration.

Not a product runtime. No CLI binary — user commands stay on `@sxo/sxo`. Does **not** own dialect Feature Matrix definitions.

## Role

| Layer | Owns |
|-------|------|
| `@sxo/harness` | R&D primitives (`feature`, `validateFeatureMatrix`, `runFeatureCase`, reporters) |
| Dialect `tests/feature-matrix/` | That dialect's capability truth source (split by `category`) |
| Dialect `tests/*.test.ts` | Vitest acceptance against the public dialect API |

Dialect **product** `src/` must not import this package. Dialects may list it as a `devDependency` only.

## Feature Matrix builders

```ts
import { feature, matrix } from '@sxo/harness';

export const arithmeticFeatures = [
  feature('Plus', 'arithmetic')
    .supported()
    .pure()
    .eval('plus.basic', '1 + 2', '3')
    .done(),
];
```

```bash
pnpm --filter @sxo/harness build
pnpm --filter @sxo/harness test
pnpm --filter @sxo/mathematica report:features
pnpm --filter @sxo/matlab report:features
```
