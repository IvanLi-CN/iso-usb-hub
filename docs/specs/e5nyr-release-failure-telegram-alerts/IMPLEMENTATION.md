# 发布失败 Telegram 告警实现状态

## 当前覆盖

- `.github/workflows/notify-release-failure.yml` 继续监听 `Release`、`Development Release` 与 `Site Publish` 的完成事件，仅处理失败结果，并排除 PR 版 `Site Publish`。
- `notify_failure` 与 `smoke_test` 均调用 `IvanLi-CN/oidrune/.github/workflows/notify.yml@e48822f99c6402a753ed86557ea029754cbab20b`。
- 调用方声明 `id-token: write`，省略 `gateway_url` / `oidc_audience` 以使用 Oidrune 默认网关，并移除 `SHOUTRRR_URL` 传递。
- 两条路径都由调用方完整生成 summary，覆盖标题、项目、状态/结果、目标 SHA、run URL，并保留 workflow、ref、attempt、actor、event 等诊断字段。
- `scripts/test-notify-release-failure-contract.mjs` 覆盖 pinned target、旧目标/secret 清除、OIDC 权限、summary 字段和触发边界，并接入现有 `check.yml`。

## 验证口径

- 本地运行 Node workflow contract test 与 actionlint。
- 本地继续执行仓库约定的 `cargo check` 与 `cargo build --release`。
- 本次迁移不触发 `workflow_dispatch` smoke，不发送真实 Telegram/Oidrune 通知，也不修改 Oidrune 控制面。

## 剩余事项

- 手动 smoke 的真实链路验证仍需由 owner 在需要时显式触发。
- GitHub Pages Actions source 的 owner 侧设置仍是独立的外部配置，不属于本迁移。
