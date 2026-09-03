# Internal WASM Runtime Artifact

**Internal implementation package. Do not install directly.**

This package contains generated TypeScript and WebAssembly material consumed by `@sxo/lite`. Application developers should install `@sxo/lite`, which owns the public API, initialization contract, browser guidance, and supported feature scope.

The artifact target is `unknown-wasm32`. Its files are produced by the repository build pipeline and may change as the WASM toolchain evolves. Do not import its internal modules, depend on its generated file layout, or publish an application API around its implementation details. If a browser build cannot locate the artifact, inspect the high-level package's bundler configuration, emitted asset URL, MIME type, worker path, and CSP before changing this package.

Maintainers should keep generated files aligned with the version consumed by `@sxo/lite`, test clean production bundles, and avoid exposing source-only files through a public npm package. This package has no independent dialect semantics, no native addon, no Jupyter transport, and no standalone symbolic API.

Report integration failures with the package manager, bundler, browser, generated artifact version, and diagnostic output. Apache License 2.0. See the repository [LICENSE](../../../../LICENSE).
