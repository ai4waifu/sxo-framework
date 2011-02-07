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
   pnpm change-logs      releases/vX.Y.Z.md     gh release edit
```

Tag 锚点见 [documentation/maintenance/tags.md](../../../documentation/maintenance/tags.md)（bump commit，非 `dev` 尖端）。

## 路径与产物

| 路径 | 用途 | 入库 |
| --- | --- | --- |
| `scripts/change-logs.mjs` | commit 索引生成器 | 是 |
| `documentation/maintenance/release-notes.template.md` | 发布稿模板 | 是 |
| `documentation/maintenance/author-github.json` | 非 noreply 邮箱 → GitHub `id` / `login` | 是 |
| `documentation/maintenance/releases/vX.Y.Z.reference.md` | 按 commit 分组的**对照稿** | **否**（gitignore） |
| `documentation/maintenance/releases/vX.Y.Z.md` | 面向用户的**发布稿** | 是 |

## ① 生成 reference

```text
pnpm change-logs --tags
pnpm change-logs --version X.Y.Z
pnpm change-logs --version X.Y.Z --write
pnpm change-logs --from vA.B.C --to vX.Y.Z
```

- `--version X.Y.Z`：范围 = 上一个 `v*` tag .. `vX.Y.Z`（semver 回退兜底）。
- `--write`：写入 `documentation/maintenance/releases/vX.Y.Z.reference.md`（与发布稿同目录）。
- 输出按 gitmoji 分组：Features / Bug Fixes / Breaking / Other；每行 `- <subject> (@user)`。

### `v0.0.0` 特例

首个 tag 无 `--from` 时，reference 覆盖**到该 tag 为止的全历史**，不能逐条照抄。只取 [tags.md](../../../documentation/maintenance/tags.md) 锚点 commit 与用户可感知的里程碑。

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

## ③ 同步 GitHub Release

**仅在用户明确要求时**：

```text
gh release edit vX.Y.Z --notes-file documentation/maintenance/releases/vX.Y.Z.md
```

## 相关入口

- [release-notes.template.md](../../../documentation/maintenance/release-notes.template.md)
- [index.md](../../../documentation/maintenance/index.md)
- [update-commit-messages](../update-commit-messages/SKILL.md)
