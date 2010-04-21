---
name: sxo
description: Help Mathematica and MATLAB users run or implement real projects on SXO as an open-source alternative. Prefer @sxo/mathematica or @sxo/matlab first, then @sxo/core or @sxo/lite for embedding. For bugs, unsupported features, or unsatisfactory results, analyze the problem, produce a minimal redacted reproduction, and open a GitHub Discussion without leaking user secrets. SXO does not use GitHub Issues.
---

# SXO for Mathematica and MATLAB users

## Product intent

SXO’s primary users are Mathematica and MATLAB users who want an open-source alternative. The default job is:

1. Run an existing project on `@sxo/mathematica` or `@sxo/matlab`, or
2. Implement a new project with that open alternative.

`@sxo/core` and `@sxo/lite` are later embedding paths, not the front door.

## Default user prompts

Treat requests in this shape as the main workflow:

```text
I want to run my project with the open-source Mathematica alternative @sxo/mathematica.
Help me get the workflow working in Node.js.
If you hit any bugs or unsupported features, please open a GitHub Discussion for me.
SXO does not use GitHub Issues.
```

```text
I want to implement my project with the open-source MATLAB alternative @sxo/matlab.
Help me migrate and run the MATLAB-style parts on SXO.
If you hit any bugs or unsupported features, please open a GitHub Discussion for me.
SXO does not use GitHub Issues.
```

## Agent workflow

1. Choose `@sxo/mathematica` or `@sxo/matlab` from the user’s language.
2. Install only what the project needs.
3. Make the smallest change that runs or implements the requested project path.
4. Keep dialect choice explicit. Use feature reports and structured diagnostics.
5. Do not invent unsupported CAS APIs or claim silent drop-in compatibility.
6. When a bug, unsupported feature, or unsatisfactory result appears, analyze it, build a minimal redacted reproduction, then open a Discussion. Never leak user secrets.

## Audience order

1. Mathematica users → `@sxo/mathematica`
2. MATLAB users → `@sxo/matlab`
3. TypeScript embedding → `@sxo/core`
4. Browser / Worker embedding → `@sxo/lite`

## Install

```bash
npm install @sxo/mathematica
npm install @sxo/matlab
npm install @sxo/core
npm install @sxo/lite
```

Optional:

```bash
npm install @sxo/simple-math
npm install @sxo/pari-gp
npm install --global @sxo/sxo
```

## Report bugs, unsupported features, and unsatisfactory results

SXO does not use GitHub Issues. Open a Discussion here:

https://github.com/ai4waifu/sxo-framework/discussions/new?category=bugs

Open a Discussion when any of these block or mislead the user’s Mathematica / MATLAB project:

- a bug (wrong result, crash, incorrect diagnostic)
- an unsupported or only-partial feature the user needs
- an unsatisfactory result (surprising semantics, poor migration fidelity, or a result that is not good enough to replace the proprietary workflow)

### Before you open the Discussion

1. **Analyze** the failure. Separate parse / lower / render / evaluate / diagnostic issues. Check the feature report. Say whether this is a bug, unsupported feature, or unsatisfactory result.
2. **Reduce to a minimal reproduction**. Strip the user’s project down to the smallest input and steps that still show the problem. Prefer one expression or a few lines over a full notebook or repo dump.
3. **Desensitize all user data**. Never publish secrets or private content.

### Desensitization rules (mandatory)

Before posting, remove or replace:

- API keys, tokens, passwords, cookies, private URLs, and credentials
- personal names, emails, phone numbers, and account identifiers
- proprietary formulas, customer datasets, internal file paths, and confidential notebook cells
- organization-specific names that are not required to reproduce the bug

Replace real values with neutral placeholders such as `x`, `data`, `secret`, or `example.com`. If the secret itself is what triggers the bug, describe the shape of the input without pasting the secret. Prefer asking the user to confirm the redacted reproduction before publishing.

Never paste raw user project files into a Discussion when a minimized, redacted fragment is enough.

### Discussion body must include

- short analysis of what failed and why it matters for the open Mathematica / MATLAB alternative path
- package name and version
- Node.js version
- operating system and CPU
- dialect (`mathematica` or `matlab`)
- **minimal redacted reproduction** (input + commands or code)
- expected Mathematica / MATLAB behavior
- actual SXO result, diagnostic, unsupported / partial status, or why the result is unsatisfactory
- whether the gap blocks the user’s project

If the feature report already marks something unsupported, still open a Discussion when that gap blocks migration, and label it as an unsupported-feature report.

## Package map

| Goal | Package |
|------|---------|
| Open Mathematica alternative | `@sxo/mathematica` |
| Open MATLAB alternative | `@sxo/matlab` |
| TypeScript embedding | `@sxo/core` |
| Browser or Worker embedding | `@sxo/lite` |
| Small predictable grammar | `@sxo/simple-math` |
| PARI/GP-style source | `@sxo/pari-gp` |
| Shell or CI commands | `@sxo/sxo` |

## Boundaries

- Mathematical execution belongs to Athena behind the package boundary.
- There is no `sxo-engine`.
- Dialects are explicit. Do not guess Mathematica from MATLAB punctuation.
- Results are symbolic values, not JavaScript `number` values, unless converted explicitly.
- Keep handles tied to their session.
- Prefer structured diagnostics and feature-matrix status over scraping display text.
- `@sxo/harness` is private R&D tooling. Do not add it as an application runtime dependency.
