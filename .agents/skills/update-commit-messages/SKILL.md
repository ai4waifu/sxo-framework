---
name: update-commit-messages
description: >-
  sxo-framework 提交信息与历史改写：gitmoji 规范、批量 `git-reword` 用法、
  `reword.pending.json` 工作流、安装 git-tools。用户提及 reword、改 commit message、
  提交规范、git-fix-message 时加载。
---

# Update Commit Messages（提交信息）

批量改历史 message 一律用全局 **`git-reword`**（[git-tools](https://github.com/oovm/git-tools)）。发布说明见 [`update-change-logs`](../update-change-logs/SKILL.md)。

## 提交规范（gitmoji）

- **subject 与 body 用英文**；skill / 维护文档正文用中文。
- subject **必须以真实 gitmoji 开头**；禁止 Conventional Commit、多个 emoji、`?` 乱码。
- subject **末尾禁止句号**；全文禁止 `;` / `；`。
- **标识符反引号**：`` `@sxo/mathematica` ``、`` `sxo-dialect-matlab` ``、`` `scripts/change-logs.mjs` `` 等。
- **版本号不进 subject**；body 勿枚举 `` `v0.0.x` `` 路径。
- 指 TypeScript 时写 **TypeScript**，勿写 bare `TS`。

### 示例

```text
📦 Publish `@sxo/*` with Mathematica `SameQ` and `Mod` lowering

📝 Add `scripts/change-logs.mjs` and maintainer release notes under `documentation/maintenance`

🔖 Bump publishable packages for Trusted Publisher tag release
```

### UTF-8（Windows）

勿用 PowerShell here-string 写 emoji commit。用 UTF-8 的 `reword.pending.json` + `git-reword`，或 `git commit -F` 指向 UTF-8 文件。

## 安装 `git-reword`

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-reword
where git-reword
```

找不到时确认 `%USERPROFILE%\.cargo\bin` 在 `PATH`。上游：<https://github.com/oovm/git-tools>。

## 批量改写

```bash
git-reword export --base <exclusive-base> --ref dev --path reword.pending.json
git-reword rewrite --base <exclusive-base> --ref dev --path reword.pending.json --dry-run
git-reword rewrite --base <exclusive-base> --ref dev --path reword.pending.json
```

`reword.pending.json` 已在 `.gitignore`。推送改写历史需用户确认后 `git push --force-with-lease origin dev`。

## 相关

- [update-change-logs](../update-change-logs/SKILL.md)
- [documentation/maintenance/index.md](../../../documentation/maintenance/index.md)

