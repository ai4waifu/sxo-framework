# SXO

TypeScript-first symbolic computation (CAS): Athena kernel · dialect crates · Node NAPI + WASM lite.

## Install (workspace)

```bash
pnpm install
pnpm build:native
pnpm build:ts
pnpm --filter @sxo/core --filter @sxo/mathematica --filter @sxo/matlab run test
```

## Packages

| Package | Role |
|---------|------|
| `@sxo/core` | Simple-math API only |
| `@sxo/simple-math` | Thin SM facade over core |
| `@sxo/mathematica` | Sole Mathematica / Wolfram dialect |
| `@sxo/lite` | Browser WASM (SM only) |
| `@sxo/sxo` | Toolchain CLI (`version` / `doctor`) |
| `@sxo/sxo-<platform>` | Native NAPI addon |

## Quick use

```typescript
import { d, simplify } from '@sxo/core';

console.log(d('x^3 + sin(x)', 'x').toString());
console.log(simplify('sin(x)^2 + cos(x)^2').toString()); // 1
```

Mathematica forms belong in `@sxo/mathematica`, not `@sxo/core`.
