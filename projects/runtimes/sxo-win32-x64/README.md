# 🪟 @sxo/sxo-win32-x64

[![license](https://img.shields.io/npm/l/@sxo/sxo-win32-x64)](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md)

`@sxo/sxo-win32-x64` is the Windows x64 native runtime artifact used by the SXO TypeScript packages. It contains a
prebuilt Node-API addon for Windows on 64-bit Intel and AMD processors. It is a platform package, not the public
symbolic-computation API. Most users should install `@sxo/sxo`, `@sxo/core`, `@sxo/mathematica`, `@sxo/matlab`, or
another high-level package and allow the package manager to select this artifact automatically.

## 👥 Who needs this package?

You normally do not need to import this package in application code. npm, pnpm, Yarn, or another compatible package
manager resolves it as an optional dependency when the host machine matches `win32` and `x64`. It exists so that a
single SXO release can ship native acceleration without forcing every user to compile Rust locally.

This package is useful when you are diagnosing installation on Windows, building an offline mirror, preparing a
reproducible enterprise cache, or inspecting which native artifact a deployment received. It can also be useful to
maintainers who are testing the native boundary independently from the TypeScript facade.

## 📦 What it contains

The package contains a file named `sxo.win32-x64-msvc.node`. The `.node` file is a dynamically loaded Node.js native
addon built for the Microsoft Visual C++ Windows toolchain and the x64 architecture. It does not contain a CLI,
TypeScript declarations, a browser bundle, or a standalone evaluator. The public API is exposed by the higher-level SXO
package that loads this addon.

The artifact participates in the following flow:

```text
your application
    -> @sxo/mathematica / @sxo/matlab / @sxo/core / @sxo/sxo
    -> optional native runtime resolution
    -> @sxo/sxo-win32-x64
    -> Athena-backed execution
```

The native addon is an implementation detail of the host adapter. It does not create a second mathematical engine and it
does not change the language semantics of a dialect package.

## 🚀 Installation

Install the package that matches your task:

```bash
pnpm add @sxo/mathematica
```

or:

```bash
npm install @sxo/sxo
```

The high-level package declares this artifact as an optional platform dependency. On a supported Windows x64 machine,
the package manager installs it during dependency resolution. Do not add this package as your only SXO dependency unless
you are intentionally testing the native loading boundary.

## 🧭 Platform contract

This artifact is constrained to:

| Property         | Required value                |
|------------------|-------------------------------|
| Operating system | Windows (`win32`)             |
| CPU architecture | 64-bit x86 (`x64`)            |
| Native format    | Node-API `.node` addon        |
| Toolchain family | MSVC                          |
| Consumer         | A compatible SXO host package |

The package is not intended for 32-bit Windows, ARM64 Windows, Linux, macOS, browsers, Deno, or a generic WebAssembly
runtime. Those environments use different artifacts or the WASM package. A Windows ARM64 machine may run x64 Node.js
under emulation, but that is a host configuration choice and is not the same as a native ARM64 build.

## 🧯 Troubleshooting

If npm reports that an optional dependency was skipped, first inspect `node --version`, `process.platform`, and
`process.arch`:

```bash
node -p "process.version + ' ' + process.platform + ' ' + process.arch"
```

Expected output includes `win32 x64`. Check that the lockfile was generated on a compatible machine or that your package
manager was not instructed to omit optional dependencies. In restricted or offline environments, populate the package
cache before installation and preserve the platform-specific entries in the lockfile.

If the high-level package cannot load the addon, reinstall dependencies after removing only the project dependency
directory and lockfile entries you intentionally regenerate. Do not copy a `.node` file from another operating system
into a Windows installation. Native binaries are not portable across operating systems, and a binary built with an
incompatible ABI can fail before JavaScript receives a useful diagnostic.

Corporate endpoint protection can also quarantine native modules. In that case, verify the package contents and
provenance through your normal supply-chain process, then allow the approved artifact according to company policy. Do
not solve a loader failure by downloading an unrelated binary from an untrusted source.

## 🚚 Deployment guidance

For CI, pin the high-level SXO package and use a lockfile. Cache dependencies using a key that includes the operating
system, architecture, Node.js major version, and lockfile hash. For packaged desktop applications, ensure the native
`.node` file remains inside the application’s packaged resources and that the loader can resolve the path at runtime.
For server deployments, install dependencies in the same platform family as the target process rather than copying
`node_modules` from another runner.

If you need a native-free deployment, choose `@sxo/lite` where its feature scope is sufficient. The WASM route is
designed for browser and portable environments, while this package is specifically for the native Node.js host path.

## 🔗 Relationship to other packages

Use `@sxo/sxo` for the command-line interface, `@sxo/mathematica` for the Wolfram-oriented frontend, `@sxo/matlab` for
the MATLAB-oriented frontend, `@sxo/simple-math` for the smallest syntax surface, and `@sxo/lite` for browser-first WASM
execution. Those packages document user-facing APIs and feature boundaries. This package documents only the platform
artifact.

## 🤝 Support and release expectations

The artifact version is released together with the SXO package family. It should not be upgraded independently from the
high-level package unless you are testing a specific release candidate or maintaining an internal mirror. Compatibility
depends on the native host ABI and the release that requested the addon.

Report reproducible loader problems with the package version, Node.js version, Windows build, architecture, package
manager, lockfile state, and the exact high-level package being imported. Avoid including proprietary source expressions
in issue reports when a minimal reproduction is sufficient.

## 📄 License

SXO is distributed under the Apache License 2.0. See the repository license file for the complete terms.

In short: install a high-level SXO package, let the package manager select this Windows x64 artifact, and treat this
package as a native distribution component rather than as a direct programming interface.

For maintainers, native loading should remain an explicit boundary. JavaScript code should use the stable facade,
diagnostics should preserve the original platform and loader context, and tests should cover both successful resolution
and the absence of this artifact on unsupported hosts. Keeping those checks at the facade boundary makes upgrades easier
and prevents platform details from leaking into dialect code.

