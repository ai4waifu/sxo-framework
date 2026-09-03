# @sxo/core

[![CI](https://img.shields.io/github/actions/workflow/status/ai4waifu/sxo-framework/ci.yml?label=CI)](https://github.com/ai4waifu/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md) [![Node.js](https://img.shields.io/badge/Node.js-%3E%3D20-339933)](https://nodejs.org/)

`@sxo/core` is the shared TypeScript integration layer for SXO. Install it when you are building an application,
library, editor integration, notebook adapter, or service that needs common SXO values, handles, sessions, diagnostics,
and runtime access without choosing a language dialect inside the core package. It is infrastructure for integrations,
not the Simple Math implementation and not a standalone computer algebra kernel.

## 🚀 Start Here

```bash
pnpm add @sxo/core
```

The package is an ESM TypeScript API with generated declarations. Import its public entry point from application code
and keep dialect-specific parsing in `@sxo/simple-math`, `@sxo/mathematica`, or `@sxo/matlab`. This explicit choice
prevents a string from being interpreted with the wrong language rules.

```ts
import {createSession} from '@sxo/core';

const session = createSession();
// Use the session and the dialect adapter selected by your application.
```

The exact helpers available in a release are documented by the generated declarations and package tests. Treat opaque
values as handles. Do not inspect them as JSON, cast them to numbers, or retain them after their session is closed.

## 👥 Who Needs Core

Choose Core when your code is another library rather than an end-user dialect. Typical users include IDE plugins,
document editors, language servers, notebook bridges, web service adapters, test harnesses, and applications that let
users choose among multiple dialects. Core gives these integrations a common place to manage runtime state and result
conversion.

Do not choose Core merely because its name sounds foundational. If you only need a small expression grammar,
`@sxo/simple-math` is easier. If you need Wolfram-style syntax, install `@sxo/mathematica`. If you need MATLAB parsing,
install `@sxo/matlab`. If you need browser execution without native addons, use `@sxo/lite` and follow its
browser-specific lifecycle.

## 🧩 Mental Model

SXO separates source language, symbolic form, and computation:

```text
application request
  -> selected dialect parser
  -> typed Form and lowering
  -> Core handles and values
  -> Athena runtime
  -> result, status, or diagnostic
```

Core does not guess the dialect and does not turn every parsed form into an executable result. A parser can accept a
construct that is only available for rendering or feature reporting. A result can be partial or resource-limited.
Preserve those distinctions in your application model.

## 🔗 Sessions and Lifetime

A session owns runtime state for a group of related operations. It may own caches, native resources, WASM state,
diagnostics, and values that refer to the same runtime. Keep session creation explicit at an application boundary such
as a request, notebook kernel, worker, or editor document.

Handles are intentionally opaque. JavaScript garbage collection does not replace session management. Closing a session
invalidates its handles, and a handle from one WASM worker cannot be assumed to work in another worker. If you serialize
work, serialize documented source or result data rather than an implementation handle.

Avoid putting handles in long-lived global caches without a corresponding session policy. When a cache entry outlives a
request, record which session owns it and define what happens when that session is closed. This is especially important
for server-side rendering, worker pools, and test suites that create many runtimes.

## 🌐 Dialect Pairing

Core is deliberately dialect-neutral. Pair it with exactly the parser and lowering package that matches your input.
Simple Math is a narrow, predictable language. Mathematica provides a defined Wolfram-style frontend surface, rendering,
feature reports, CLI, and Jupyter helpers. MATLAB provides a MATLAB syntax frontend and migration-oriented feature
reporting.

Do not claim that Core provides Mathematica or MATLAB compatibility. Compatibility belongs to a documented dialect
feature set and its current backend support. Use feature matrices before exposing a feature in your own product.

## 🩺 Results and Diagnostics

Use structured result objects and diagnostics in program logic. Display text is for people, logs, and notebooks.
Diagnostic codes and fields are better for automation because localized messages can change. Keep source locations when
presenting parser or lowering errors.

Your integration should distinguish successful evaluation, partial evaluation, unsupported syntax, invalid input,
cancellation, resource exhaustion, and runtime failure. Do not convert all of them to `undefined` or a generic string. A
caller needs to know whether retrying, changing dialect, increasing a budget, or editing the input is appropriate.

## 💻 Native and Browser Environments

Core is primarily used by Node.js-facing packages. Native optional dependencies are selected by the high-level package
and must match the operating system, CPU, Node.js ABI, and produced binary. Do not import a platform `.node` package as
a general API.

For browser applications, use `@sxo/lite`. Its WASM runtime has separate loading, memory, worker, and bundler
constraints. Do not assume that a Node session can be moved into a browser or that a native handle can be transferred
across workers.

## 🔌 API Design Advice

Keep your application domain types separate from SXO handles. Convert at the edge and log both your own identifier and
the session identifier when a result can come from multiple runtimes. Make dialect selection part of request
configuration. Keep rendered text separate from canonical or structured result data.

When wrapping Core, expose cancellation and resource limits in your own API. Do not create an unbounded queue of
expressions. Close sessions in deterministic lifecycle hooks, and make restart behavior visible to users. If a runtime
becomes unhealthy, fail with a diagnostic and create a new session rather than reusing stale handles.

## 🧪 Testing

Test the integration path: create a session, parse through the selected dialect, lower, evaluate a representative form,
inspect the result status, and close the session. Add negative tests for malformed input, unsupported features, stale
handles, cancellation, and resource limits. Run Node and browser packages in their actual target environments.

## 🚧 Non-Goals

Core is not a second computation engine, a parser for every supported language, a replacement for native or WASM runtime
packages, or a promise of complete Mathematica/MATLAB compatibility. It does not make arbitrary user input safe to
evaluate without application-level limits and isolation.

## 📌 Release Notes

SXO is in the `0.0.x` stage. Pin versions in applications, read changelogs, and rerun dialect feature reports after
upgrades. Keep `exports`, declarations, and runtime imports aligned with the installed version.

## 📄 License

Apache License 2.0. See the repository [LICENSE](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md). Report
security issues privately through the configured project contact.

