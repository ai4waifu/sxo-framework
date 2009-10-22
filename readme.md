# SXO

[![CI](https://img.shields.io/github/actions/workflow/status/ai4waifu/sxo-framework/ci.yml?label=CI)](https://github.com/ai4waifu/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md) [![Node.js](https://img.shields.io/badge/Node.js-%3E%3D20-339933)](https://nodejs.org/) [![TypeScript](https://img.shields.io/badge/TypeScript-first-3178C6)](https://www.typescriptlang.org/)

SXO is a TypeScript-first symbolic-computation product layer for JavaScript applications. It provides dialect-aware
parsing, symbolic forms, lowering, rendering, structured diagnostics, and dependable symbolic execution. The repository
contains public packages, browser WebAssembly distribution, platform-selected native artifacts, command-line tools,
notebook adapters, and development workflows.

SXO is not a replacement for Mathematica, MATLAB, or another complete commercial environment. Dialect packages
understand a defined language surface and translate it into SXO forms. The SXO runtime provides the computation
boundary. This separation lets applications use familiar syntax without turning partial syntax support into an incorrect
compatibility promise.

## 🧭 Choose a Package

| Goal                            | Package            | Audience                                          |
|---------------------------------|--------------------|---------------------------------------------------|
| Shell, CI, or scripted commands | `@sxo/sxo`         | Node.js automation users                          |
| TypeScript integration          | `@sxo/core`        | Library and application authors                   |
| Small predictable grammar       | `@sxo/simple-math` | Examples, education, tests                        |
| Wolfram-style source            | `@sxo/mathematica` | Wolfram and notebook users                        |
| MATLAB-style source             | `@sxo/matlab`      | MATLAB users and tooling authors                  |
| PARI/GP-style number theory     | `@sxo/pari-gp`     | Placeholder package reserved for a future adapter |
| Browser or worker execution     | `@sxo/lite`        | Frontend and bundler users                        |

Platform packages are optional native artifacts selected by npm. The internal WASM artifact is consumed by `@sxo/lite`,
not installed directly. The homepage is a private site application.

## ⚡ First Result

```bash
pnpm add @sxo/simple-math
```

```ts
import {d, simplify} from '@sxo/simple-math';

console.log(d('x^3 + sin(x)', 'x').toString());
console.log(simplify('sin(x)^2 + cos(x)^2').toString());
```

Results are symbolic values, not JavaScript `number` values. Preserve their structure or cross to machine numbers
explicitly. For shell workflows:

```bash
npm install --global @sxo/sxo
sxo --help
```

Use the CLI for pipelines and automation. Use TypeScript when you need structured results, stable diagnostics, or
application-controlled lifetime management.

## 🛠️ Workspace Development

```bash
pnpm install
pnpm build:native
pnpm build:ts
pnpm test:js
pnpm lint
```

Native and WASM builds require the toolchains documented by their scripts. Run focused package tests before the complete
build.

## 🧩 Architecture

```text
source text or TypeScript form
        -> dialect parser and Form
        -> SXO lowering and rendering
        -> Athena value and execution contract
        -> structured result or diagnostic
```

The TypeScript layer owns ergonomics, forms, handles, conversion, diagnostics, and dialect syntax. Athena owns
mathematical execution. Native packages provide platform-specific Node bindings. `@sxo/lite` provides the browser WASM
route. There is no second `sxo-engine` layer between SXO and Athena.

A parser may recognize a construct that the runtime cannot evaluate. A renderer may preserve a form without claiming
semantic equivalence. Feature matrices report these states explicitly so applications can warn, defer, or choose another
package.

## 🌐 Dialects

`@sxo/simple-math` is the smallest entry point for examples, teaching tools, tests, and lightweight applications. Its
narrow grammar is deliberate.

`@sxo/mathematica` supports a defined Wolfram-style frontend surface, rendering, feature reporting, the `wolframscript`
command, and Jupyter helpers. It does not promise complete Mathematica compatibility or provide a Wolfram kernel.

`@sxo/matlab` is a MATLAB syntax frontend. It parses supported source, lowers it to SXO/Athena forms, renders it, and
reports unsupported or partial features. It is not a MATLAB runtime or toolbox implementation.

## 🚀 Native, WASM, and Jupyter

Use native Node packages for the broadest supported Node behavior, CLI integration, and notebook helpers. Optional
dependencies select an OS and CPU artifact. Test clean containers because a development machine may have libraries that
production does not.

Use `@sxo/lite` for browser-first deployments, workers, and bundlers that cannot install native addons. WASM has
different startup, memory, transfer, and debugging characteristics. Initialize it asynchronously and keep long
operations away from the UI thread.

Use the Mathematica package's Jupyter helper for notebook protocol integration. A kernelspec is an adapter around SXO.
It does not make unsupported language features execute.

## 🔗 Sessions and Handles

Treat a session as the owner of runtime state. Expressions, numeric values, caches, and native or WASM resources may be
associated with it. Handles are opaque references into managed state. They are not serialized syntax trees, JavaScript
numbers, or stable memory addresses.

Keep handles alive while their session is alive. Do not use a handle after closing its session, move handles between
independent WASM instances, or retain native references without the package API. JavaScript garbage collection is not
the same as runtime resource reclamation.

## 🩺 Diagnostics and Feature Reports

Prefer structured diagnostics over matching display text. Diagnostics can identify an operation, source location,
unsupported feature, domain mismatch, resource limit, cancellation, or runtime failure. Localized messages may change
with locale, while codes and structured fields are intended for automation.

Use feature-matrix commands or subpath exports before promising behavior. Supported, partial, parse-only, render-only,
unsupported, and not-applicable are different states and should remain different in application code.

## 📦 Deployment

Pin Node.js and package-manager versions for native deployments. Test every target OS and CPU on a clean host. For
browser builds, verify WASM asset handling, worker URLs, CSP headers, base paths, and offline behavior. Never import
`@sxo/lite-unknown-wasm32` directly.

Apply authentication, request-size, time, memory, and cancellation limits around services that evaluate user input.
Long-lived runtimes should not be exposed to untrusted users without an explicit resource policy.

## 🚚 Release Notes for Maintainers

SXO is in the `0.0.x` stage. APIs, package boundaries, diagnostics, and feature coverage may evolve before a stable
contract is declared. Pin versions, read changelogs, and rerun the feature report relevant to your workflow after
upgrades.

Publishable packages are released through the trusted publisher workflow. Local publishing is not the normal release
path. Workspace dependency ranges are replaced by release automation before publication.

## 🤝 Contribution

Keep syntax, lowering, rendering, and runtime changes in their owning package. When adding a dialect feature, update its
form, lowering behavior, rendering expectation, diagnostics, and feature matrix. When adding a runtime feature, document
Node and WASM availability, resource behavior, and session lifetime.

Report the package, version, runtime, operating system, dialect, input category, and diagnostic code with bug reports.
Keep examples runnable and avoid claiming compatibility that tests do not establish.

## 🔒 Security and License

Treat source expressions, notebook input, and CLI input as untrusted data. Add your own authentication, authorization,
resource, and cancellation policy around evaluation services. Report security issues privately through the repository's
configured security contact.

SXO is distributed under the Apache License 2.0. See [LICENSE](LICENSE). Third-party packages and generated native or
WASM artifacts may carry additional notices.

## 🧰 Practical Workflow Patterns

For a command-line report, keep source input in a file, select the dialect explicitly, capture standard output and
diagnostics separately, and record the package version in the report metadata. This makes a failed build reproducible
and prevents a display change from silently breaking a pipeline. For a service, create a session per tenant or per
isolation boundary, enforce a request budget, and return structured status to the caller. Do not let an HTTP request
hold an unbounded queue of expressions in one process.

For an editor integration, parse early, preserve the original source span, and show partial or unsupported status beside
the source rather than hiding it behind a generic error. Keep the rendered form separate from the editable source. A
renderer is allowed to choose stable notation for display while the parser continues to own source-language details.

For a notebook, initialize once per kernel session, keep output values associated with the cell execution that produced
them, and surface diagnostics with source locations. Restarting a kernel invalidates its handles. A notebook extension
should therefore detect session changes and clear stale references instead of retrying them indefinitely.

For a browser application, load WASM during an explicit initialization phase, show loading and failure states, and move
expensive work to a worker. If the application needs a feature that is only available through native Node bindings, say
so in the product design and provide a server or precomputation path. Do not make a browser bundle depend on an
accidental native fallback.

## ✅ What SXO Guarantees

SXO guarantees package-level contracts, not universal language compatibility. A package documents the input forms it
accepts, the result categories it returns, its runtime requirements, and the diagnostics it can produce. The common
TypeScript layer keeps these contracts consistent across dialects. Athena supplies the mathematical runtime behind the
supported operations.

SXO also guarantees that package boundaries are explicit. Simple Math syntax is not silently interpreted as Wolfram
Language. MATLAB frontend behavior is not presented as MATLAB execution. Native and WASM packages identify their
different operational constraints. This makes it possible to build reliable tooling even while feature coverage expands.

## 🚧 What SXO Does Not Guarantee

SXO does not guarantee that every expression accepted by a dialect is executable, that every renderer is reversible, or
that a familiar function name has identical semantics across languages. It does not guarantee the performance profile of
native and WASM on every workload. It does not provide a general sandbox for arbitrary untrusted programs. It does not
replace authentication, authorization, cancellation, request limits, or deployment observability.

These limits are intentional. Honest boundaries let users compose SXO with their own application policies and let the
project improve individual dialect features without changing the identity of the underlying mathematical result.

## 🧯 Troubleshooting Checklist

When installation fails, check Node.js version, package-manager version, operating system, CPU architecture, and whether
optional dependencies were omitted. When a native addon fails to load, test the high-level package on a clean host and
inspect the diagnostic before importing any platform artifact directly. When a browser build fails, inspect the emitted
WASM URL, worker path, CSP, and bundler asset configuration.

When parsing fails, confirm that the input belongs to the selected dialect and consult its feature matrix. When
evaluation returns a partial result, preserve the status and diagnostic code. When a handle becomes invalid, check
whether its session or worker was restarted. When output differs after an upgrade, compare package versions, dialect
selection, runtime mode, and feature-report status before comparing display strings.

## 🗂️ Repository Layout

The repository keeps product packages under `projects/runtimes`, the private homepage under `projects/homepage`, and
build, test, and release automation under `scripts`. Package READMEs live next to their manifests so npm users see the
same guidance as repository contributors. Native artifacts are intentionally separated from TypeScript sources. The root
manifest is a private workspace manifest and is not itself an installable SXO product.

## 🔭 Related Projects

- [Runtime packages](projects/runtimes)
- [Homepage application](projects/homepage)
- [Build scripts](scripts)
- [Athena](../athena.rs)

Use the README inside the package you selected for exact imports, supported commands, constraints, and troubleshooting.

## 🧠 Design Principles

The first principle is explicit translation. Source syntax enters through a dialect parser and becomes a typed Form.
Lowering is a visible step, so a caller can inspect or report what happened before asking Athena to execute it. This
prevents a parser from quietly inventing backend semantics and gives tooling a stable place to attach source spans,
warnings, and migration hints.

The second principle is separation of display and identity. Rendering is for people, logs, notebooks, and generated
source. It is not a substitute for a canonical value identity. Applications that cache or compare results should use
structured values and documented conversion methods. They should not hash pretty-printed output or assume that two
equivalent displays imply the same evaluation context.

The third principle is honest status. SXO operations can return complete results, partial results, unsupported-feature
diagnostics, resource failures, cancellations, or parse failures. These states are deliberately visible. A user-facing
application may choose to retry, simplify the request, show a warning, or ask for a different dialect, but the package
should not erase the distinction to make a demo look successful.

The fourth principle is runtime portability. Native Node bindings are valuable for server and desktop integrations. WASM
is valuable for browsers, workers, and constrained deployments. Neither is declared universally superior. The package
selection should follow the deployment environment, workload size, startup budget, and available operating-system
facilities. Tests should run in the environment where the application will ship.

The fifth principle is managed lifetime. Symbolic expressions can share substructure and can outlive one request while
remaining tied to a session. Opaque handles and runtime-managed resources exist to make that sharing explicit.
Applications should close sessions, release worker resources, and avoid treating a runtime pointer as a portable value.
Serialization should use a documented wire or text format, not an implementation object snapshot.

The sixth principle is capability-aware growth. A feature report is part of the API, not an afterthought. It gives users
a map of what is implemented, what is planned, and what belongs to another package. This is particularly important for
Wolfram and MATLAB users, who may arrive with expectations shaped by mature environments. SXO can be useful without
claiming that every language construct or toolbox behavior is already present.

## 🔌 API Integration Advice

When integrating `@sxo/core`, keep your own domain model above the common handles and values. Adapt application
identifiers to SXO handles at the boundary and convert back at the boundary. This keeps a database row, editor document,
or job identifier from being confused with a runtime object identifier. Include the session identifier in logs when a
result can be produced by more than one runtime instance.

When integrating a dialect, expose the dialect choice in configuration rather than guessing from punctuation. A user who
selects Wolfram-style input should see Wolfram diagnostics and feature status. A user who selects MATLAB-style input
should receive MATLAB frontend diagnostics. If an application accepts multiple languages, maintain separate parse entry
points and make the selected language visible in the request record.

When integrating the CLI, treat exit status and diagnostic output as part of the automation contract. Keep
human-readable output for terminals and structured output for tools where the CLI supports it. Avoid scraping a
localized sentence to decide whether an operation succeeded. In CI, retain the command, package version, Node.js
version, and relevant feature report as build artifacts.

When integrating the browser package, make initialization a first-class state in the UI. Show loading progress where
possible, show a useful failure action, and cancel work when a view or route is destroyed. A worker can own a runtime
and return serialized result data to the main thread. Do not pass opaque handles between workers unless the package
explicitly documents that transfer.

When integrating Jupyter, treat the kernelspec installation as deployment configuration. Record where the kernelspec was
installed, which Node.js executable it references, and which package versions it uses. A notebook can be opened on a
different machine or container, so avoid assuming that a local global npm installation exists everywhere. Provide a
clear kernel restart path when a native module or runtime session becomes unhealthy.

## 🧪 Testing Strategy

Package tests should cover the user journey rather than only individual helper functions. Start with installation or
build checks, then parse a representative expression, lower it, render it, and assert the structured result or
diagnostic category. Add negative cases for unsupported syntax, malformed input, domain mismatch, resource limits,
cancellation, and stale handles.

Dialect tests should preserve source spans and verify feature-matrix status. A test that only checks a rendered string
can miss a lowering bug. A test that only checks parser acceptance can miss an execution boundary. Keep a small set of
golden examples for documentation and a broader generated or property-based set for expression normalization where
appropriate.

Runtime tests should run on native and WASM targets where the package promises both. Check clean installation, optional
dependency selection, worker initialization, and production bundler output. For platform packages, verify that the
high-level package selects the artifact and that an unsupported platform produces an actionable diagnostic. Do not turn
a platform artifact into a public API test target.

## ✍️ Documentation Maintenance

Every README should answer the same practical questions in the order a new user encounters them: what problem the
package solves, whether it matches the user's environment, how to install it, how to obtain a first result, how the
mental model works, what is supported, what is intentionally absent, and how to diagnose a failure. Keep
package-specific details in package READMEs and repository-wide architecture in this file.

When behavior changes, update examples and feature reports together with code. When a package becomes public, add
accurate npm metadata, repository links, keywords, license information, and publication policy. When a package remains
private or internal, say so prominently and avoid badges that imply a supported direct-install workflow. Documentation
is part of the compatibility contract because users make package and deployment decisions from it.

## 🗺️ Final Orientation

If you need the smallest successful symbolic expression, begin with `@sxo/simple-math`. If you need a familiar language
surface, choose the dialect package that matches the source and read its feature report. If you need a shared
integration layer, use `@sxo/core`. If you need a shell or notebook workflow, use the CLI or Jupyter helper with the
documented Node.js requirements. If you need browser execution, choose `@sxo/lite` and plan for asynchronous WASM
initialization. If npm is selecting a platform artifact, let the high-level package own that decision.

This repository is designed to grow by making each boundary more useful without making the boundaries less truthful.
Syntax can become richer, Athena can execute more domains, diagnostics can become more actionable, and native or WASM
packaging can improve independently. The user-facing promise remains simple: choose the package that matches your
workflow, inspect the documented capability, keep runtime lifetime explicit, and treat partial behavior as information
rather than as silent success.

The same rule applies to examples in issues, pull requests, blog posts, and application documentation. Name the package,
dialect, runtime, and feature status. A compact example is valuable when a reader can run it and understand why it
works. A large compatibility claim is harmful when it hides the exact input forms, backend capabilities, or deployment
assumptions behind it. SXO documentation therefore favors reproducible paths, explicit limits, and links to the package
that owns the next decision.

Maintainers should also keep the npm page useful to someone who has never opened the repository. Package descriptions
should be searchable but precise, exports should match the files actually shipped, and badges should communicate CI,
license, runtime, and publication status without suggesting that an internal artifact is a supported public API. The
result should help a reader make a confident installation choice in a few minutes and then provide enough detail to
build a responsible integration over the longer term.
