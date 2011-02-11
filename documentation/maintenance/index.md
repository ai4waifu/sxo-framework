# Maintenance

Release workflow for `@sxo/*` npm packages: tag anchors, change-log references, and agent skills.

## npm releases

- Workflow: [`.agents/skills/update-change-logs/SKILL.md`](../../.agents/skills/update-change-logs/SKILL.md) (reference → release notes → `gh release edit`)
- Commit messages and `git-reword`: [`.agents/skills/update-commit-messages/SKILL.md`](../../.agents/skills/update-commit-messages/SKILL.md)
- Template: [release-notes.template.md](release-notes.template.md)
- Tag anchors: [tags.md](tags.md)
- Draft index: `git-change-logs --version X.Y.Z` (`pnpm change-logs` shortcut, requires global `git-change-logs`)
- `--write` → `releases/vX.Y.Z.reference.md` (gitignored). `git-change-logs lookup --email` for `author-github.json`
- Author email map: [author-github.json](author-github.json)
