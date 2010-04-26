# 🤖 @sxo/skills

## AI4Science · agentic-friendly · TypeScript-first

`@sxo/skills` helps research agents and scientists use SXO for AI4Science work. Install once; the skill then handles
package choice, honest coverage, Discussion reporting, and data redaction automatically. Your prompts only need to
state the scientific goal.

Mathematica and MATLAB dialects are optional. Use them when you already know those languages and want to keep working
in a familiar syntax. They are not required for every project.

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
I am doing AI4Science work. Help me implement this scientific workflow with SXO in Node.js.
```

```text
I am doing AI4Science work. I already know MATLAB, so use @sxo/matlab where it helps.
Help me run this scientific project on SXO.
```

```text
I am doing AI4Science work. I already know Mathematica, so use @sxo/mathematica where it helps.
Help me run this scientific project on SXO.
```

## Embedding prompts

```text
Help me embed this SXO scientific workflow in a TypeScript service with @sxo/core.
```

```text
Help me run this SXO scientific workflow in the browser with @sxo/lite.
```

## What the skill does automatically

After installation, the agent should:

- Optimize for the scientific outcome first.
- Use Mathematica/MATLAB dialects only when the user is familiar with them or asks for that syntax.
- Prefer `@sxo/core` / `@sxo/lite` for TypeScript-native embedding.
- On bugs, unsupported features, or unsatisfactory results: analyze, minimize, redact, and open a Discussion.
- Never use GitHub Issues for SXO bugs.
- Never publish secrets, personal data, proprietary formulas, unpublished results, or confidential notebooks.

You do not need to repeat those instructions in every prompt.

## Open a Discussion

https://github.com/ai4waifu/sxo-framework/discussions/new?category=bugs
