# @sxo/sxo

[![CI](https://img.shields.io/github/actions/workflow/status/vm-z/sxo-framework/ci.yml?label=CI)](https://github.com/vm-z/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../../../LICENSE) [![Node.js](https://img.shields.io/badge/Node.js-%3E%3D20-339933)](https://nodejs.org/)

`@sxo/sxo` is the SXO command-line package for Node.js automation. Use it in shell scripts, CI jobs, documentation builds, local experiments, and notebook support workflows where a command and diagnostic output are more convenient than writing an integration first.

## Install

```bash
npm install --global @sxo/sxo
sxo --help
```

The CLI requires Node.js 20 or newer. Read `--help` in the installed version for the available commands and options. Keep the selected dialect explicit. The CLI is a host for SXO packages, not a standalone mathematical kernel.

## Automation

Use files or standard input for repeatable jobs. Capture output and diagnostics separately when your CI system supports it. Record the package version, Node.js version, dialect, locale, and runtime mode in artifacts. Automation should branch on exit status and structured diagnostic codes rather than matching localized sentences.

## Dialects

Install the dialect package that matches your source: `@sxo/simple-math`, `@sxo/mathematica`, or `@sxo/matlab`. A familiar function name does not guarantee identical semantics across languages. A successful parse does not guarantee executable backend support.

## Native Runtime

Native optional dependencies are selected by npm according to operating system and CPU. Test clean containers and production hosts. If an addon cannot load, inspect the diagnostic and decide whether to stop, use a supported fallback, or move the operation to `@sxo/lite`.

## Non-Goals

This package is not a replacement for Mathematica, MATLAB, or a general shell sandbox. Add your own authentication, resource, time, cancellation, and isolation policy around services that evaluate untrusted input.

## Versioning and License

SXO is `0.0.x`. Pin versions and review changes before upgrades. Apache License 2.0. See [LICENSE](../../../../LICENSE).
