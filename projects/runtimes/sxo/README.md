# @sxo/sxo

[![CI](https://img.shields.io/github/actions/workflow/status/ai4waifu/sxo-framework/ci.yml?label=CI)](https://github.com/ai4waifu/sxo-framework/actions) [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md) [![Node.js](https://img.shields.io/badge/Node.js-%3E%3D20-339933)](https://nodejs.org/)

`@sxo/sxo` is the SXO command-line package for Node.js automation. Use it in shell scripts, CI jobs, documentation
builds, local experiments, and notebook support workflows where a command and diagnostic output are more convenient than
writing an integration first.

## 🚀 Install

```bash
npm install --global @sxo/sxo
sxo --help
```

The CLI requires Node.js 20 or newer. Read `--help` in the installed version for the available commands and options.
Keep the selected dialect explicit. The CLI is a host for SXO packages, not a standalone mathematical kernel.

## ⚙️ Automation

Use files or standard input for repeatable jobs. Capture output and diagnostics separately when your CI system supports
it. Record the package version, Node.js version, dialect, locale, and runtime mode in artifacts. Automation should
branch on exit status and structured diagnostic codes rather than matching localized sentences.

## 🌐 Dialects

Install the dialect package that matches your source: `@sxo/simple-math`, `@sxo/mathematica`, or `@sxo/matlab`. A
familiar function name does not guarantee identical semantics across languages. A successful parse does not guarantee
executable backend support.

## 💻 Native Runtime

Native optional dependencies are selected by npm according to operating system and CPU. Test clean containers and
production hosts. If an addon cannot load, inspect the diagnostic and decide whether to stop, use a supported fallback,
or move the operation to `@sxo/lite`.

## 🚧 Non-Goals

This package is not a replacement for Mathematica, MATLAB, or a general shell sandbox. Add your own authentication,
resource, time, cancellation, and isolation policy around services that evaluate untrusted input.

## 📌 Package status

SXO is currently in early development. Pin the package family in automation and review the installed command contract
before upgrades.

## 🧭 A Typical Command

```bash
sxo --help
sxo version
sxo doctor
```

Start with `--help` because the installed release is the command authority. In CI, check the exit status first and
retain diagnostics separately from human-readable output.

## 🧰 Choosing the Next Package

The CLI is a host, not a language guesser. Pair it with Simple Math, Mathematica, or MATLAB according to the actual
input. Use Core when a TypeScript application needs sessions and structured values directly.

## 🧯 Troubleshooting

When the command is missing, verify Node.js and the npm bin directory. When native loading fails, check OS, CPU, ABI,
and optional dependencies. When parsing fails, inspect the selected dialect and its feature report. Add time, memory,
and cancellation limits around untrusted input.

## 🧭 Selecting a frontend

The CLI is a host and delivery surface, not a new language implementation. Use `@sxo/mathematica` for Wolfram-oriented
source, `@sxo/matlab` for MATLAB source, and `@sxo/simple-math` for the deliberately small grammar. Use
`@sxo/core` when a TypeScript application needs shared handles and types, and choose `@sxo/lite` for browser-first WASM
delivery. Installing this package does not imply that every dialect is enabled in every environment.

## 🔁 Execution model

An SXO request travels through a frontend parser, a dialect-specific form, lowering, the Athena public facade,
evaluation, and rendering. Syntax and presentation belong to SXO. Mathematical semantics belong to Athena. This keeps a
CLI workflow conceptually aligned with Node and browser integrations without inventing a second evaluator for the
command line.

Short-lived intermediate values belong to an evaluation session. Only values that escape that session become persistent
results. The CLI manages this boundary, so application scripts should treat returned values as API results and should
never assume access to temporary internal buffers.

## 🧪 Automation practices

For CI, pin the package version with a lockfile and use Node.js 20 or newer. Use process exit status as the success
signal and preserve stderr for diagnostics. Do not parse localized prose as a machine protocol. When source comes from a
user, pass it as a file or safely quoted argument rather than constructing an unescaped shell command.

Native optional dependencies are selected by the package manager. Install dependencies inside the target container or
runner, and cache by operating system, architecture, Node version, and lockfile hash. For a native-free deployment, use
`@sxo/lite` when its feature scope is sufficient rather than copying a `.node` file between operating systems.

## 🛠️ Diagnostics and locales

Diagnostics identify the failing stage: parsing, lowering, evaluation, rendering, or host integration. Preserve
diagnostic codes and source spans when forwarding errors. Locale files affect presentation and must not alter
mathematical meaning. If a translation is missing, retain the stable code and original context instead of replacing the
error with a generic exception.

## 🚧 Scope and non-goals

This package is not a drop-in replacement for Mathematica, MATLAB, or Wolfram Engine. It does not promise complete
language compatibility, identical numerical algorithms, or identical output formatting. It is not a license manager for
external products. External comparison programs must be installed and licensed by the user and are outside normal
package installation and CI.

## 🤝 Development and support

Useful issue reports include package version, Node.js version, operating system, architecture, command line, minimal
source expression, exit status, and complete diagnostic code. Remove confidential expressions. Contributions should test
the correct boundary: dialect behavior belongs in a dialect package, shared mathematical behavior belongs in Athena, and
process or CLI formatting behavior belongs here.

## 📦 Release expectations

## 🧾 Reproducible command records

For every automated invocation, keep the exact command line, selected dialect, input file checksum, locale, Node.js
version, operating system, CPU architecture, and diagnostic code. This small record turns a vague “the CLI changed”
report into a reproducible case. It also lets a team compare native and WASM runs without confusing a parser difference
with a platform-loader difference.

The package is published through the project release workflow, with platform artifacts resolved as optional
dependencies. Keep the lockfile in reviewable changes and test a clean installation on every supported operating system.
A release may add syntax or improve diagnostics without claiming compatibility with an external language. Read the
feature matrix and release notes when upgrading production scripts. Stable diagnostic identifiers are more reliable
integration points than the exact wording of a localized message, and a minimal reproducible command is the best
starting point for support.

For long-running jobs, record the command, selected dialect, package version, and environment alongside the output. This
makes failures reproducible when native modules or locale settings differ between developers and CI runners.

## 📄 License

SXO is distributed under the [Apache License 2.0](https://github.com/ai4waifu/sxo-framework/blob/dev/License.md).

