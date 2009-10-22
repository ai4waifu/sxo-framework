# @sxo/simple-math

[![CI](https://img.shields.io/github/actions/workflow/status/ai4waifu/sxo-framework/ci.yml?label=CI)](https://github.com/ai4waifu/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md) [![TypeScript](https://img.shields.io/badge/TypeScript-first-3178C6)](https://www.typescriptlang.org/)

`@sxo/simple-math` is the smallest SXO entry point for predictable symbolic expressions. It is designed for teaching
tools, examples, tests, documentation demos, small utilities, and applications that need a compact expression surface
without importing a large dialect. Install it when your users want familiar arithmetic and elementary symbolic
operations, and when a deliberately narrow grammar is more valuable than broad language compatibility.

## 🚀 Install

```bash
pnpm add @sxo/simple-math
```

```ts
import {d, simplify} from '@sxo/simple-math';

const derivative = d('x^3 + sin(x)', 'x');
const identity = simplify('sin(x)^2 + cos(x)^2');

console.log(derivative.toString());
console.log(identity.toString());
```

The returned objects represent symbolic values. They are not JavaScript numbers. Preserve the symbolic result when exact
structure matters and use an explicit conversion boundary when your application deliberately accepts machine-number
behavior.

## 💡 Why Simple Math Exists

Many applications do not need a complete Wolfram or MATLAB frontend. They need a small grammar that is easy to explain,
test, and embed. Simple Math keeps that promise by avoiding implicit dialect guessing and by using the shared SXO core
for values, diagnostics, and runtime integration.

The package is a facade over the common API. It does not introduce a second mathematical engine and it does not silently
enable Mathematica syntax. If your product needs Wolfram names, held forms, patterns, parts, or notebook integration,
move to `@sxo/mathematica`. If it needs MATLAB source analysis, use `@sxo/matlab`.

## 🧭 User Journey

Start with one expression and one operation. Add variables only when the first result is understandable. Keep source
text in your own document model and retain the returned symbolic value for display or a later operation. When input is
invalid, show the diagnostic near the source instead of replacing it with a generic parse error.

For an editor, parse on demand and preserve source spans. For a teaching application, show both the original expression
and the rendered result. For a test fixture, assert structured behavior where possible and reserve string snapshots for
display contracts. For a service, add request size, time, memory, and cancellation limits around evaluation.

## 📐 Grammar and Boundaries

The grammar is intentionally smaller than a commercial computer algebra language. Do not assume that every function
name, indexing form, assignment, control construct, or formatting convention from another language is accepted. A
familiar spelling can have different semantics in different dialects.

Use feature documentation and package tests as the authority for supported input. Unsupported syntax should remain
visible as unsupported. A parser accepting a form does not automatically mean that the backend can evaluate every
operation represented by that form.

## 💻 Node and Browser Use

Simple Math is appropriate for Node.js applications through the normal SXO runtime packages. Browser applications should
choose `@sxo/lite` when they need a native-free WASM route. Do not transfer opaque handles between unrelated sessions or
workers. JavaScript garbage collection is not a substitute for closing a runtime session.

## 🩺 Errors and Diagnostics

Use structured diagnostics in automation. Localized display text can change with locale, while codes and fields are
intended to remain useful for branching, logging, and support. Distinguish malformed input, unsupported syntax, partial
results, cancellation, resource exhaustion, and runtime failure.

## 🚧 Non-Goals

This package is not a MATLAB runtime, not a Wolfram kernel, not a general parser for every mathematical notation, and
not a promise of exact compatibility with another language. It is a small SXO dialect facade with an honest surface.

## 🧪 Testing and Release Notes

Run the package tests with Vitest and include both successful expressions and negative grammar cases. SXO is currently
`0.0.x`, so pin versions and read changelogs before upgrades. Keep examples aligned with the feature surface shipped by
the installed version.

## 📌 Package status

## 🧪 Examples That Age Well

Prefer examples with one operation and one expected symbolic shape. Keep negative examples beside syntax users might
confuse with Wolfram Language or MATLAB. Assert structured results where possible and use text snapshots only for
display contracts.

## 🧭 When to Graduate

Move to Mathematica for Wolfram forms and notebook workflows, MATLAB for MATLAB source, Core for integration
infrastructure, and Lite for browser-first execution. Simple Math remains valuable because it does not make those
choices implicitly.

## 🧯 Troubleshooting

A parse error usually means the input is outside the narrow grammar. Preserve the source span and diagnostic. Keep
runtime failure, unsupported syntax, cancellation, and resource exhaustion as distinct application states.

## 🧠 A useful mental model

Simple Math is a small frontend that turns readable expressions into structured symbolic values. It is intentionally
closer to a dependable expression language than to a complete scientific computing environment. The parser owns spelling
and precedence, the lowering step gives the shared Athena runtime a language-neutral representation, and the renderer
turns the result back into a form suitable for a terminal, test, or application view. This separation means you can
build a small editor or teaching tool without copying the evaluator into JavaScript.

The package does not silently reinterpret unsupported syntax. If an input looks like a construct from another dialect,
the correct result is a diagnostic identifying the unsupported form. That behavior is useful in production because a
typo cannot quietly become a different mathematical operation. It is also useful in education because the boundary of
the language remains visible.

## 📋 Choosing an integration style

| Need                          | Recommended path          |
|-------------------------------|---------------------------|
| Tiny Node script              | Import `@sxo/simple-math` |
| Shared handles and sessions   | Add `@sxo/core`           |
| Wolfram-oriented syntax       | Use `@sxo/mathematica`    |
| MATLAB source                 | Use `@sxo/matlab`         |
| Browser without native addons | Use `@sxo/lite`           |

Keep the source dialect explicit in user interfaces. A dropdown, file extension, or command-line option is preferable to
guessing from punctuation. If an application accepts multiple dialects, preserve the selected dialect in logs and
serialized task descriptions so the same expression can be reproduced later.

## 🔒 Boundaries for applications

Do not expose internal evaluator buffers as application state. A result returned by the public API should be treated as
a stable value or handle according to the API contract. Temporary evaluation objects are session-local and may be
reclaimed after the operation. This matters most in editors that retain results in a history list: store the public
result, not an implementation reference.

For untrusted expressions, configure the surrounding application with time, memory, and input-size limits. A small
grammar reduces the attack surface but does not eliminate the need for resource policy. Report cancellation and resource
exhaustion separately from a syntax error so callers can decide whether to retry, explain the limit, or ask the user to
simplify the expression.

## 🧪 Testing strategy

Tests should cover the complete user journey: accepted source, structured result, rendering, and rejected source with a
stable diagnostic. Include whitespace, precedence, unary signs, nested expressions, and malformed input. Keep examples
short enough that a failure points to one language rule. If you snapshot rendered text, also assert structural values so
a formatting change cannot hide a semantic regression.

## 🤝 Support and contribution

When reporting a problem, include the package version, Node.js version, source expression, selected dialect, expected
result, actual result, and diagnostic code. Do not include confidential source. New syntax should be proposed with a
clear user need, examples, negative cases, and a statement of whether it belongs in Simple Math or another dialect.
Mathematical behavior shared by all dialects belongs in Athena rather than in this package.

## 📦 Release expectations

The package is in the `0.0.x` stage and does not promise compatibility with older experimental APIs. Pin versions in
applications, read release notes before upgrading, and run the package test suite in CI. The Apache License 2.0 governs
this package. Simple Math is deliberately modest: its value is a clear, predictable path from a small expression to a
shared symbolic runtime.

## 📄 License

SXO is distributed under the [Apache License 2.0](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md).

