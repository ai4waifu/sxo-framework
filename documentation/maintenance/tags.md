# Version tag anchors

`v0.0.x` tags point at **version-boundary commits** (bump or semver alignment), not `dev` tip. `pnpm change-logs --version X.Y.Z` uses range `previous tag .. this tag`.

| Tag | Commit | Anchor |
| --- | --- | --- |
| `v0.0.0` | `bb4fbf5` | npm publish pipeline and `wolframscript` CLI |
| `v0.0.1` | `cfc8b05` | require green CI before npm publish |
| `v0.0.2` | `df0ea1f` | align `@sxo/*` manifests to the `0.0.x` line |
| `v0.0.3` | `b8ccc2b` | bump workspace package versions for release |
| `v0.0.4` | `5d6f3aa` | bump publishable packages for Trusted Publisher tag release |
| `v0.0.5` | `12d2dd1` | bump publishable packages for Trusted Publisher tag release |
| `v0.0.6` | `3227b83` | bump publishable packages for Trusted Publisher tag release |
| `v0.0.7` | `96997da` | bump publishable packages to `0.0.7` (SameQ/Mod matrix work in range) |

Re-apply tags locally (coordinate before `--force` push):

```text
git tag -f v0.0.0 bb4fbf5
git tag -f v0.0.1 cfc8b05
git tag -f v0.0.2 df0ea1f
git tag -f v0.0.3 b8ccc2b
git tag -f v0.0.4 5d6f3aa
git tag -f v0.0.5 12d2dd1
git tag -f v0.0.6 3227b83
git tag -f v0.0.7 96997da
```
