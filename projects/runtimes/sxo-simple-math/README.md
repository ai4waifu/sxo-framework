# @sxo/simple-math

[![CI](https://img.shields.io/github/actions/workflow/status/vm-z/sxo-framework/ci.yml?label=CI)](https://github.com/vm-z/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../../../LICENSE) [![TypeScript](https://img.shields.io/badge/TypeScript-first-3178C6)](https://www.typescriptlang.org/)

`@sxo/simple-math` is the smallest SXO entry point for predictable symbolic expressions. It is designed for teaching tools, examples, tests, documentation demos, small utilities, and applications that need a compact expression surface without importing a large dialect. Install it when your users want familiar arithmetic and elementary symbolic operations, and when a deliberately narrow grammar is more valuable than broad language compatibility.

## Install

```bash
pnpm add @sxo/simple-math
```

```ts
import { d, simplify } from '@sxo/simple-math';

const derivative = d('x^3 + sin(x)', 'x');
const identity = simplify('sin(x)^2 + cos(x)^2');

console.log(derivative.toString());
console.log(identity.toString());
```

The returned objects represent symbolic values. They are not JavaScript numbers. Preserve the symbolic result when exact structure matters and use an explicit conversion boundary when your application deliberately accepts machine-number behavior.

## Why Simple Math Exists

Many applications do not need a complete Wolfram or MATLAB frontend. They need a small grammar that is easy to explain, test, and embed. Simple Math keeps that promise by avoiding implicit dialect guessing and by using the shared SXO core for values, diagnostics, and runtime integration.

The package is a facade over the common API. It does not introduce a second mathematical engine and it does not silently enable Mathematica syntax. If your product needs Wolfram names, held forms, patterns, parts, or notebook integration, move to `@sxo/mathematica`. If it needs MATLAB source analysis, use `@sxo/matlab`.

## User Journey

Start with one expression and one operation. Add variables only when the first result is understandable. Keep source text in your own document model and retain the returned symbolic value for display or a later operation. When input is invalid, show the diagnostic near the source instead of replacing it with a generic parse error.

For an editor, parse on demand and preserve source spans. For a teaching application, show both the original expression and the rendered result. For a test fixture, assert structured behavior where possible and reserve string snapshots for display contracts. For a service, add request size, time, memory, and cancellation limits around evaluation.

## Grammar and Boundaries

The grammar is intentionally smaller than a commercial computer algebra language. Do not assume that every function name, indexing form, assignment, control construct, or formatting convention from another language is accepted. A familiar spelling can have different semantics in different dialects.

Use feature documentation and package tests as the authority for supported input. Unsupported syntax should remain visible as unsupported. A parser accepting a form does not automatically mean that the backend can evaluate every operation represented by that form.

## Node and Browser Use

Simple Math is appropriate for Node.js applications through the normal SXO runtime packages. Browser applications should choose `@sxo/lite` when they need a native-free WASM route. Do not transfer opaque handles between unrelated sessions or workers. JavaScript garbage collection is not a substitute for closing a runtime session.

## Errors and Diagnostics

Use structured diagnostics in automation. Localized display text can change with locale, while codes and fields are intended to remain useful for branching, logging, and support. Distinguish malformed input, unsupported syntax, partial results, cancellation, resource exhaustion, and runtime failure.

## Non-Goals

This package is not a MATLAB runtime, not a Wolfram kernel, not a general parser for every mathematical notation, and not a promise of exact compatibility with another language. It is a small SXO dialect facade with an honest surface.

## Testing and Versioning

Run the package tests with Vitest and include both successful expressions and negative grammar cases. SXO is currently `0.0.x`, so pin versions and read changelogs before upgrades. Keep examples aligned with the feature surface shipped by the installed version.

## License

Apache License 2.0. See [LICENSE](../../../../LICENSE).
