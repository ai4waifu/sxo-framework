# @sxo/matlab

[![CI](https://img.shields.io/github/actions/workflow/status/vm-z/sxo-framework/ci.yml?label=CI)](https://github.com/vm-z/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../../../LICENSE)

`@sxo/matlab` is a MATLAB syntax frontend for SXO. It helps MATLAB users and tooling authors parse supported source, inspect forms, lower expressions to the SXO/Athena contract, render results, and review feature coverage during migration. It is not a MATLAB runtime, toolbox implementation, or MATLAB-compatible kernel.

## Install

```bash
pnpm add @sxo/matlab
```

The package is an ESM TypeScript API for Node.js workflows. Use the feature matrix export and report command before exposing a construct in a product.

## Workflow

```text
MATLAB-style source -> parser -> Form -> lowering -> Athena request
```

Parsing, lowering, rendering, and execution are separate steps. A construct can be recognized for migration analysis while remaining unsupported for execution. Keep source locations and diagnostics when presenting that distinction.

## Scope

The package owns MATLAB-facing syntax and frontend behavior. Athena owns mathematical execution. Do not assume that MATLAB functions, indexing, assignment, scripts, toolboxes, or runtime side effects are available merely because a name parses. Unsupported behavior should be reported explicitly.

## Feature Reports

Use the feature matrix to distinguish supported, partial, parse-only, render-only, unsupported, and not-applicable features. Store the report with migration artifacts so users can see which changes are required. Do not turn a feature percentage into a compatibility claim.

## Diagnostics and License

Use structured diagnostic codes in automation and localized text only for display. Add resource limits around services evaluating user input. SXO is `0.0.x`; pin versions. Apache License 2.0. See [LICENSE](../../../../LICENSE).
