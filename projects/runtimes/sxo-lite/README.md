# @sxo/lite

[![CI](https://img.shields.io/github/actions/workflow/status/ai4waifu/sxo-framework/ci.yml?label=CI)](https://github.com/ai4waifu/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md) [![WASM](https://img.shields.io/badge/runtime-WebAssembly-654FF0)](https://webassembly.org/)

`@sxo/lite` is the browser-first SXO distribution for TypeScript applications that cannot install native Node addons. It
is intended for web pages, Web Workers, bundlers, static sites, and selected edge-style environments. It consumes the
internal WASM implementation package and exposes a small TypeScript-facing surface for symbolic work.

## 🚀 Install and Initialize

```bash
pnpm add @sxo/lite
```

Import the public package from your application. Keep initialization asynchronous because the WASM module and its linear
memory must be prepared before evaluation. The exact exported helpers follow the generated declarations shipped with the
package.

## 🌐 Browser Workflow

Load the runtime during an explicit application phase, show loading and failure states, and move expensive expressions
into a Web Worker. A worker should own one runtime instance and return structured result data to the page. Do not move
opaque handles between workers or after a runtime restart.

WASM has different startup, memory, transfer, and debugging costs from native Node execution. Measure the workload in
the browsers you support. Budget linear memory in addition to ordinary page memory, and cancel work when a route or
editor view is destroyed.

## 🎯 Scope

Lite is deliberately narrower than the Node distribution. It does not provide native `.node` addons, the `wolframscript`
command, or Jupyter/ZMQ transport. Choose a dialect package only when its browser support is documented. For Wolfram or
MATLAB frontend work that requires Node-only helpers, run the dialect on a server or use the corresponding native
package.

## 📦 Bundlers

Verify how your bundler handles the WASM asset, worker URL, base path, CSP, and production output. Test a clean
production build rather than relying on a development server. If an asset is emitted beside JavaScript, configure the
deployment to serve it with the correct MIME type and cache policy.

## 🩺 Diagnostics and Limits

Preserve structured diagnostics and distinguish loading failure, parse failure, unsupported feature, cancellation, and
resource exhaustion. Apply your own request-size and time limits around user input. A browser package is not a security
sandbox for arbitrary programs.

## 🚧 Non-Goals

This package is not a general-purpose browser CAS, not a native runtime wrapper, and not a guarantee that Node, WASM,
and commercial dialects have identical feature coverage. Read the feature report and package tests for the exact
release.

## 📌 Package status

SXO is in the `0.0.x` stage. Pin versions and retest production bundling after upgrades.

## 🧭 Which Browser Apps Fit Lite

Lite fits calculator editors, educational notebooks, documentation demos, migration previews, and worker-backed symbolic
tools. It is not the right path when a workflow requires native Node integration or an unsupported dialect feature.

## 🧵 Worker Pattern

```ts
const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
worker.postMessage({ source: 'x^2 + 1' });
```

Let the worker own the runtime and return serializable data. Render loading, success, partial, cancellation, and failure
states independently.

## 🧯 Troubleshooting

Check the emitted WASM URL, MIME type, CSP, worker base path, and production asset handling. Measure startup and peak
linear memory in supported browsers. Upgrade Lite and its internal artifact together.

## 🧭 Decide whether Lite fits

Lite is the right first choice when your application runs in a browser, a Web Worker, a static deployment, or an
environment where native Node addons cannot be installed. It is also a useful option for documentation examples and
interactive teaching tools because the user can try an expression without a compiler toolchain or operating-system
package. It is not automatically the best choice for every workload. A long-running server that needs native host
integration, Jupyter support, or the broadest available runtime surface should begin with the Node packages instead.

| Situation                     | Choice                          |
|-------------------------------|---------------------------------|
| Browser bundle or static site | `@sxo/lite`                     |
| Web Worker evaluation         | `@sxo/lite` with a worker entry |
| Node CLI or server            | `@sxo/sxo` or `@sxo/core`       |
| Wolfram-oriented frontend     | `@sxo/mathematica`              |
| MATLAB frontend               | `@sxo/matlab`                   |

## 🧱 What happens at runtime

Lite exposes a TypeScript-friendly boundary while the internal WASM artifact supplies the compiled host implementation.
Your application owns page state, loading UI, cancellation affordances, and rendering. The runtime owns symbolic
execution and returns public values or diagnostics. Keep those responsibilities separate so a failed WASM fetch is
presented as a deployment problem while a rejected expression remains a language diagnostic.

```mermaid
flowchart LR
  A[Browser application] --> B[@sxo/lite]
  B --> C[WASM artifact]
  C --> D[Athena-backed execution]
  D --> E[Value or diagnostic]
  E --> A
```

The internal package is deliberately not the API that application code should import. Importing `@sxo/lite` leaves room
for the artifact layout, loader details, and generated bindings to change without forcing every browser application to
know those details.

## ⚡ Startup and user experience

WASM startup can be visible on a slow device or a cold browser cache. Show an explicit loading state, avoid blocking the
first paint, and begin evaluation only after the runtime reports readiness. Cache static assets according to your
deployment policy and include the WASM file in the same release as the JavaScript wrapper. A stale JavaScript wrapper
paired with a newer binary can produce confusing loader errors.

Use a small first expression to verify the complete path. Then allow the user to submit larger work with a busy
indicator and cancellation control. Keep the input available after failure so the user can correct it instead of
starting again. For editor applications, debounce parsing separately from evaluation and never assume that an
intermediate keystroke is a complete expression.

## 🌐 Browser deployment

Bundlers must preserve the generated WASM asset and emit a URL that the loader can resolve in production. Test a built
deployment rather than relying only on a development server. Check the response MIME type, content security policy, base
path, cache headers, and worker URL. Static hosting systems often place assets under a path different from local
development. Record the final public asset URL in a smoke test.

For Web Workers, keep the runtime and its message protocol inside the worker. Send source text or structured public
values across the boundary, not internal handles or references to temporary memory. Define messages for ready, result,
diagnostic, cancellation, and failure. Terminate a worker only after pending work is resolved or explicitly abandoned,
and create a new worker when the runtime contract requires a fresh session.

## 📦 Bundle and performance guidance

Measure the experience that matters to users: time to ready, time to first successful evaluation, peak memory, and
evaluation latency for representative expressions. Do not infer browser performance from a native benchmark. Avoid
repeatedly creating runtimes in a hot interaction loop. Reuse a ready session when the API permits it, and release
application references to results that are no longer displayed.

Lite does not promise that every native feature or every dialect integration is available in the browser. Consult the
feature matrix and treat unsupported capability as a normal product state. Feature detection should happen at startup or
when selecting a workflow, not after a user has entered a long expression.

## 🔐 Security and isolation

Expressions may come from users or remote documents. Apply input-size, time, and memory limits at the application
boundary. Keep the browser origin and worker policy deliberate, and do not use `eval` around SXO source. A content
security policy should permit only the assets and worker locations your deployment actually needs. Do not treat
successful parsing as authorization to access application data: symbolic evaluation and application permissions are
separate concerns.

## 🧪 Testing checklist

Test both development and production builds. Cover a missing WASM asset, a wrong base path, a blocked worker, malformed
source, unsupported syntax, cancellation, and a successful result. Run smoke tests in the browser versions your users
support and include a low-memory device when possible. Assert structured result behavior in addition to rendered strings
so a presentation change cannot hide a semantic regression.

## 🚧 Scope and non-goals

Lite is not a browser version of Mathematica or MATLAB, and it does not provide Jupyter or ZMQ. It is a portable SXO
host path with the feature scope published for the release. The package does not expose the internal Rust crates, and it
must not become a second mathematical engine. Shared mathematical behavior belongs to Athena, while browser loading and
worker behavior belong to this adapter.

## 🤝 Support and releases

When reporting an issue, include package version, browser and version, bundler, deployment URL shape, operating system,
whether a worker is used, the network response for the WASM asset, and a minimal source expression. Keep the `@sxo/lite`
and internal WASM artifact versions aligned. Pin versions in applications, test a clean production build, and review
release notes before upgrading. The package is distributed under the Apache License 2.0.

## 📄 License

SXO is distributed under the [Apache License 2.0](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md).

