---
name: update-change-logs
description: >-
  sxo-framework 版本发布说明闭环：生成 commit 级 reference、手工提炼
  `documentation/maintenance/releases/vX.Y.Z.md`、按需 `gh release edit` 同步 GitHub。
  用户提及 change-logs、release notes、更新 changelog、GitHub Release 时加载。
---

# Update Change Logs（发布说明）

在本仓，**更新 changelog** 指 **reference → 发布稿 →（可选）GitHub Release** 的完整闭环，不是把 reference 直接发布，也不是写「正式发布 npm」等元信息。

```text
① 生成 reference  →  ② 提炼发布稿  →  ③ 同步 GitHub Release（用户明确要求时）
   git-change-logs     releases/vX.Y.Z.md     gh release edit
```

Tag 锚点见 [documentation/maintenance/tags.md](../../../documentation/maintenance/tags.md)（bump commit，非 `dev` 尖端）。

## Prerequisite: install `git-change-logs`

Same source as `git-reword` — [git-tools](https://github.com/oovm/git-tools):

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-change-logs
```

Verify with `git-change-logs --help`. This repo's `pnpm change-logs` is a shortcut and **requires** the binary on `PATH`.

## 路径与产物

| 路径 | 用途 | 入库 |
| --- | --- | --- |
| `git-change-logs` (git-tools global CLI) | commit index generator | no |
| `documentation/maintenance/release-notes.template.md` | release notes template | yes |
| `documentation/maintenance/author-github.json` | non-noreply email → GitHub `id` / `login` | yes |
| `documentation/maintenance/releases/vX.Y.Z.reference.md` | per-commit **reference** draft | **no** (gitignore) |
| `documentation/maintenance/releases/vX.Y.Z.md` | user-facing **release notes** | yes |

## ① 生成 reference

From the repo root (or any subdirectory):

```text
git-change-logs --tags
git-change-logs --version X.Y.Z
git-change-logs --version X.Y.Z --write
git-change-logs --from vA.B.C --to vX.Y.Z
```

(`pnpm change-logs …` is equivalent when the binary is installed.)

- `--version X.Y.Z`: range = previous `v*` tag .. `vX.Y.Z` (semver fallback).
- `--write`: writes `documentation/maintenance/releases/vX.Y.Z.reference.md`.
- Output grouped by gitmoji: Features / Bug Fixes / Breaking / Other. One bullet per commit: `- <subject> (@user)`.

### `v0.0.0` edge case

When there is no `--from`, the first tag spans **all history** to that tag. Do not paste every line into release notes. Keep milestones aligned with [tags.md](../../../documentation/maintenance/tags.md).

## ② 提炼发布稿

**必读**：`documentation/maintenance/release-notes.template.md` 与刚生成的 `vX.Y.Z.reference.md`。

**完成标准**：

1. 编辑 `documentation/maintenance/releases/vX.Y.Z.md`（**不是** `.reference.md`）。
2. 保留模板全部分类；无内容写 `None`。
3. **发布稿正文用英文**（GitHub Release 公开面）；只写读者安装 `@sxo/*` 后能感知的变化。
4. `## 👥 Contributors`：从 reference 复制头像墙。

### 提炼规则（硬性）

| reference 内容 | 发布稿 |
| --- | --- |
| Release / publish / bump 版本 | **跳过** |
| CI、reword、Biome/fmt 门禁 | **跳过**（或 Other 写 `None`） |
| 方言新 lowering / Feature Matrix 能力 | 合并为 1 条英文 |
| 同主题多条 commit | 合并为 1 条 |

**禁止**：reference 整段粘贴、hash 清单、「正式发布 npm」空话。

### 标题 emoji

发布稿首行**固定**：

```markdown
# 🚀 vX.Y.Z
```

🐛 / ✨ / 🔧 只出现在正文小节标题里。

## Contributor email map

`documentation/maintenance/author-github.json` — same shape as valkyrie.rs (`id` + `login`).

When the contributor wall is empty, resolve handles before re-running `--write`:

```text
git-change-logs lookup --email you@example.com
git-change-logs lookup --login handle
```

Noreply GitHub emails are parsed automatically. For other emails, set `GITHUB_TOKEN` and use `--fetch` if needed.

## ③ 同步 GitHub Release

**仅在用户明确要求时**：

```text
gh release edit vX.Y.Z --notes-file documentation/maintenance/releases/vX.Y.Z.md
```

## 相关入口

- [release-notes.template.md](../../../documentation/maintenance/release-notes.template.md)
- [index.md](../../../documentation/maintenance/index.md)
- [update-commit-messages](../update-commit-messages/SKILL.md)
- git-tools docs: [change-logs.md](https://github.com/oovm/git-tools/blob/dev/documentation/change-logs.md)
