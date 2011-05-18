# Maintenance

Release workflow for `@sxo/*` npm packages: version tags, change-log references, and agent skills.

## npm releases

- Workflow: [`.agents/skills/update-change-logs/SKILL.md`](../../.agents/skills/update-change-logs/SKILL.md) (reference → release notes → `gh release edit`)
- Commit messages and `git-reword`: [`.agents/skills/update-commit-messages/SKILL.md`](../../.agents/skills/update-commit-messages/SKILL.md)
- Template: [release-notes.template.md](release-notes.template.md)
- Version tags: `git tag -l 'v*'` and `git-change-logs --tags` (annotated tags on publish-ready commits, not `dev` tip)
- Draft index: `git-change-logs --version X.Y.Z` (`pnpm change-logs` shortcut, requires global `git-change-logs`)
- `--write` → `releases/vX.Y.Z.reference.md` (gitignored). `git-change-logs lookup --email` for `author-github.json`
- Author email map: [author-github.json](author-github.json)
