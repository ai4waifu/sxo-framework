# @sxo/mathematica

[![CI](https://img.shields.io/github/actions/workflow/status/vm-z/sxo-framework/ci.yml?label=CI)](https://github.com/vm-z/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../../../LICENSE)

`@sxo/mathematica` is the SXO Wolfram Language frontend. It is for users who need Wolfram-style source parsing, symbolic forms, rendering, feature reports, the `wolframscript` command, or Jupyter installation helpers. It is not a complete Mathematica implementation and it is not a Wolfram kernel.

## Install

```bash
pnpm add @sxo/mathematica
```

Use the package from Node.js 20 or newer. Native optional dependencies are selected by npm when the runtime needs them. Read the feature matrix before promising a language construct in an application.

## What It Owns

This package owns Wolfram-facing syntax and forms, including the documented behavior of names, held expressions, patterns, parts, rendering, and lowering. The Athena runtime owns mathematical execution. Parsing, lowering, rendering, and evaluation are separate statuses: accepting source does not prove that every backend operation is available.

```ts
import { parse } from '@sxo/mathematica';

const form = parse('Hold[x^2 + 1]');
console.log(form.toString());
```

The exact API follows the installed declarations. Preserve structured forms and diagnostics instead of scraping display strings.

## CLI and Jupyter

The package provides a `wolframscript` entry point and a Jupyter helper export. A kernelspec is an adapter around SXO and does not provide full Mathematica compatibility. Record the Node.js executable, package version, and installation location in notebook deployment logs.

## Compatibility Boundaries

Use feature reports to distinguish supported, partial, parse-only, render-only, unsupported, and not-applicable behavior. Do not infer semantics from familiar names. Unsupported constructs should remain visible to users and should not be silently lowered to a different operation.

## Diagnostics, Testing, and License

Use diagnostic codes and fields in automation. Test parsing, lowering, rendering, evaluation boundaries, feature reports, and negative cases. SXO is `0.0.x`; pin versions and review changes. Apache License 2.0. See [LICENSE](../../../../LICENSE).
