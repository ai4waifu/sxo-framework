# 🤖 @sxo/skills

## Give your AI coding agent an SXO toolkit

`@sxo/skills` is for Mathematica and MATLAB users who want an open-source alternative. Install it, then ask your agent
to run or implement your project on `@sxo/mathematica` or `@sxo/matlab`. When something is wrong or unsupported, the
agent should help you open a GitHub Discussion. SXO does not use GitHub Issues.

## Start in seconds

```bash
npx @sxo/skills
```

Extra flags are forwarded to the skills CLI:

```bash
npx @sxo/skills -g
npx @sxo/skills -a cursor -y
```

## Copy your first prompt

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

## Later embedding prompts

```text
I already use @sxo/mathematica or @sxo/matlab for my project.
Help me embed the same workflow in a TypeScript service with @sxo/core.
If you hit any bugs or unsupported features, please open a GitHub Discussion for me.
```

```text
I already use @sxo/mathematica or @sxo/matlab for my project.
Help me run the same workflow in the browser with @sxo/lite.
If you hit any bugs or unsupported features, please open a GitHub Discussion for me.
```

## What the skill helps your agent do

- Treat Mathematica and MATLAB users as the primary audience.
- Prefer `@sxo/mathematica` or `@sxo/matlab` before `@sxo/core` or `@sxo/lite`.
- Help the user run or implement a real project, not only a demo snippet.
- Keep coverage honest through feature reports and structured diagnostics.
- Open a Discussion for bugs, unsupported features, and unsatisfactory results. Never use GitHub Issues.
- Before posting: analyze the failure, reduce it to a minimal reproduction, and desensitize all user data.
- Never publish secrets, credentials, personal data, proprietary formulas, or confidential notebook content.

## Open a Discussion

https://github.com/ai4waifu/sxo-framework/discussions/new?category=bugs

Required content:

- short analysis
- package, version, Node.js, OS/CPU, dialect
- minimal **redacted** reproduction
- expected Mathematica / MATLAB behavior
- actual SXO result, unsupported status, or why the result is unsatisfactory

Do not paste the user’s raw project. Replace secrets and private values with placeholders before publishing.
