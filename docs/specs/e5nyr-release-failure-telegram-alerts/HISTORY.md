# 发布失败 Telegram 告警历史

## 2026-07-07

- 新增 repo-local notifier wrapper，覆盖 Release、Development Release 与非 PR Site Publish 失败。
- 保留 `workflow_dispatch` smoke test 入口，并通过 `SHOUTRRR_URL` 调用旧的共享 Telegram reusable workflow。

## 2026-09-01

- 实时确认 Oidrune latest release `v0.1.14`，并将目标 pinned 到 `e48822f99c6402a753ed86557ea029754cbab20b`。
- 将失败与 smoke 两条调用迁移到 Oidrune `notify.yml`，改为传递 `outcome` 与 caller-generated `summary`。
- 补齐 caller 的 `id-token: write` 权限，移除旧 `github-workflows` target 与 `SHOUTRRR_URL` secret 传递，并保留 Site Publish 非 PR 过滤语义。
- 新增并接入 workflow contract test，覆盖 pinned SHA、summary、权限、过滤和 smoke 路径。
