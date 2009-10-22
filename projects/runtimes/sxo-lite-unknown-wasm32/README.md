# Internal WASM Runtime Artifact

**Internal implementation package. Do not install directly.**

This package contains generated TypeScript and WebAssembly material consumed by `@sxo/lite`. Application developers
should install `@sxo/lite`, which owns the public API, initialization contract, browser guidance, and supported feature
scope.

The artifact target is `unknown-wasm32`. Its files are produced by the repository build pipeline and may change as the
WASM toolchain evolves. Do not import its internal modules, depend on its generated file layout, or publish an
application API around its implementation details. If a browser build cannot locate the artifact, inspect the high-level
package's bundler configuration, emitted asset URL, MIME type, worker path, and CSP before changing this package.

Maintainers should keep generated files aligned with the version consumed by `@sxo/lite`, test clean production bundles,
and avoid exposing source-only files through a public npm package. This package has no independent dialect semantics, no
native addon, no Jupyter transport, and no standalone symbolic API.

Report integration failures with the package manager, bundler, browser, generated artifact version, and diagnostic
output. Apache License 2.0. See the repository [LICENSE](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md).

## 🧭 Why this package exists

The browser adapter needs a distribution unit that can carry generated bindings and a WebAssembly payload independently
from the TypeScript facade. This package provides that unit. It lets the build pipeline target `unknown-wasm32` and lets
`@sxo/lite` resolve the resulting files as one versioned dependency. The separation is operational, not a new product
boundary: browser users get a stable Lite API, while maintainers can update generated bindings and artifact layout
behind that API.

```mermaid
flowchart LR
  A[Athena and adapter build] --> B[WASM compilation]
  B --> C[generated bindings and artifacts]
  C --> D[@sxo/lite-unknown-wasm32]
  D --> E[@sxo/lite]
  E --> F[Browser or Web Worker application]
```

The artifact must remain semantically boring. It does not own a dialect, parser policy, evaluation API, session
contract, or browser UI. Those responsibilities belong to the appropriate high-level package or shared engine.

## 🚫 Do not install directly

Application developers should install:

```bash
npm install @sxo/lite
```

Do not add this package directly to an application, import its generated modules, or write code against filenames under
`dist`, `lib`, or `src`. Direct use couples an application to build internals that can change without notice. It can
also bypass the loader, initialization sequence, feature checks, and diagnostics that `@sxo/lite` provides.

The package is private by design. If it appears in a lockfile, that usually means the high-level Lite package selected
it transitively. A direct dependency request is a signal to review the application manifest rather than an instruction
to expose this artifact as a public API.

## 🧱 Artifact contract

| Area          | Contract                                     |
|---------------|----------------------------------------------|
| Consumer      | `@sxo/lite` only                             |
| Target        | `unknown-wasm32`                             |
| Runtime       | Browser or compatible WASM host through Lite |
| Semantics     | Provided by Athena and the SXO adapter       |
| Public API    | None                                         |
| Native addon  | None                                         |
| Jupyter/ZMQ   | None                                         |
| Compatibility | Coupled to the consuming Lite release        |

Generated JavaScript and declarations are glue. They should not be mistaken for a stable TypeScript library. The
package’s stable requirement is that the consuming Lite version can load its artifact and communicate with it according
to the build contract.

## 🛠️ Maintainer workflow

When changing the WASM boundary, build the artifact through the repository script and then build Lite against the
produced result. Run a clean production bundle, not only a development server. Check the emitted URL, response MIME
type, worker path, base path, CSP, and cache behavior. A generated file that works from a local filesystem can still
fail after a static host rewrites asset paths.

Keep generated outputs from the same source revision and toolchain. Do not mix a binding generated from one commit with
a binary generated from another. Lock the compiler and wasm-bindgen versions where the project requires reproducibility,
and record toolchain changes in release notes for maintainers.

## 🔄 Version alignment

The artifact and `@sxo/lite` are a coordinated pair. Upgrade them together, test their package-manager resolution, and
verify that a clean install does not retain an old binary in a cache. If the release workflow publishes
platform-independent artifacts separately, the workflow must still enforce matching versions or an explicit
compatibility range.

Do not use the internal package to bypass a broken public release. Fix the build or release metadata at the high-level
package boundary so users receive a coherent dependency graph. Trusted publishing and release automation should publish
only the artifacts intended for publication and should not turn this internal package into a surprise public API.

## 🌐 Browser and worker integration

Lite decides how and when the artifact is initialized. Keep browser application state outside this package and send only
public data across worker messages. Do not retain pointers, generated module internals, or temporary evaluator
references after a request. Loading, ready, result, diagnostic, cancellation, and failure are application-visible states
managed by Lite.

If a browser cannot locate the artifact, inspect the high-level bundle first. Verify that the generated asset is
included, that its URL is correct after deployment, and that the server supplies an appropriate WASM MIME type. Check
CSP and worker restrictions before changing generated code. Most failures at this stage are packaging or deployment
problems, not mathematical problems.

## 🧪 Required verification

Maintainer checks should include successful resolution through `@sxo/lite`, missing-artifact behavior, malformed asset
behavior, wrong base paths, worker loading, a minimal evaluation, a diagnostic response, and a production static
deployment. Test at least one browser representative of each supported deployment class. Measure time to ready and first
evaluation separately from steady-state execution.

Do not use this package as a substitute for Athena numeric or algorithm tests. Shared mathematical correctness belongs
in the Athena repository. This package should verify ABI glue, generated bindings, artifact loading, and integration
behavior.

## 🔐 Security and supply chain

Treat generated binaries as release artifacts subject to normal provenance, review, and integrity controls. Do not
download an unrelated WASM file to repair a local failure. Verify the lockfile, package source, build revision, and
publishing workflow. Browser applications still need their own input-size, timeout, memory, and cancellation policies
because a generated artifact is not a security sandbox.

## 🚧 Compatibility boundaries

This artifact does not promise native Node support, complete dialect coverage, Mathematica or MATLAB compatibility,
Jupyter transport, or a standalone evaluator. It cannot be used to infer the full feature scope of `@sxo/lite`; the
public package and feature documentation define that scope. The `0.0.x` series allows generated layouts to change.

## 🤝 Issue reporting

Include the consuming Lite version, internal artifact version, package manager, lockfile, bundler, browser, deployment
host, public asset URL shape, HTTP response headers, worker usage, and complete console diagnostic. Report whether the
failure occurs before initialization, during loading, or during evaluation. Avoid posting proprietary source when a
minimal expression reproduces the problem.

## 📄 License

The artifact is distributed under the Apache License 2.0. Its internal status means that its file layout and generated
bindings are implementation details, not that it carries a different license or a direct user-facing contract.


