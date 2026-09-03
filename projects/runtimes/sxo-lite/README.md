# @sxo/lite

[![CI](https://img.shields.io/github/actions/workflow/status/vm-z/sxo-framework/ci.yml?label=CI)](https://github.com/vm-z/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../../../LICENSE) [![WASM](https://img.shields.io/badge/runtime-WebAssembly-654FF0)](https://webassembly.org/)

`@sxo/lite` is the browser-first SXO distribution for TypeScript applications that cannot install native Node addons. It is intended for web pages, Web Workers, bundlers, static sites, and selected edge-style environments. It consumes the internal WASM implementation package and exposes a small TypeScript-facing surface for symbolic work.

## Install and Initialize

```bash
pnpm add @sxo/lite
```

Import the public package from your application. Keep initialization asynchronous because the WASM module and its linear memory must be prepared before evaluation. The exact exported helpers follow the generated declarations shipped with the package.

## Browser Workflow

Load the runtime during an explicit application phase, show loading and failure states, and move expensive expressions into a Web Worker. A worker should own one runtime instance and return structured result data to the page. Do not move opaque handles between workers or after a runtime restart.

WASM has different startup, memory, transfer, and debugging costs from native Node execution. Measure the workload in the browsers you support. Budget linear memory in addition to ordinary page memory, and cancel work when a route or editor view is destroyed.

## Scope

Lite is deliberately narrower than the Node distribution. It does not provide native `.node` addons, the `wolframscript` command, or Jupyter/ZMQ transport. Choose a dialect package only when its browser support is documented. For Wolfram or MATLAB frontend work that requires Node-only helpers, run the dialect on a server or use the corresponding native package.

## Bundlers

Verify how your bundler handles the WASM asset, worker URL, base path, CSP, and production output. Test a clean production build rather than relying on a development server. If an asset is emitted beside JavaScript, configure the deployment to serve it with the correct MIME type and cache policy.

## Diagnostics and Limits

Preserve structured diagnostics and distinguish loading failure, parse failure, unsupported feature, cancellation, and resource exhaustion. Apply your own request-size and time limits around user input. A browser package is not a security sandbox for arbitrary programs.

## Non-Goals

This package is not a general-purpose browser CAS, not a native runtime wrapper, and not a guarantee that Node, WASM, and commercial dialects have identical feature coverage. Read the feature report and package tests for the exact release.

## Versioning and License

SXO is in the `0.0.x` stage. Pin versions and retest production bundling after upgrades. Apache License 2.0. See [LICENSE](../../../../LICENSE).
