# 发布失败 Telegram 告警接入

## Context and Scope

仓库通过 `.github/workflows/notify-release-failure.yml` 为 `Release`、`Development Release` 与 `Site Publish` 提供发布/部署失败告警，并保留 repo-local 的手动 smoke test 入口。

本主题只定义 notifier sidecar 与 Oidrune reusable workflow 的调用契约，不改动发布逻辑、版本策略或产物内容，也不新增第二套通知渠道。调用目标固定为 `IvanLi-CN/oidrune/.github/workflows/notify.yml@e48822f99c6402a753ed86557ea029754cbab20b`，由调用方提供 OIDC 权限和完整 summary。

失败通知覆盖 `Release`、`Development Release` 以及非 PR 的 `Site Publish`；普通 PR CI 不在通知范围内。手动 `workflow_dispatch` 仅用于发送 smoke summary，本地与 PR 验证不得触发真实通知。

## Requirements

### REQ-E5NYR-001: Failure event scope

The notifier MUST listen for completed `Release`, `Development Release`, and `Site Publish` workflow runs, and MUST notify only when the conclusion is `failure`. It MUST exclude `Site Publish` runs whose event is `pull_request`.

### REQ-E5NYR-002: Pinned Oidrune target

Both notification jobs MUST call `IvanLi-CN/oidrune/.github/workflows/notify.yml@e48822f99c6402a753ed86557ea029754cbab20b`. No reusable workflow call may use `@main`.

### REQ-E5NYR-003: OIDC permission

The caller MUST provide `id-token: write` permission to the reusable workflow jobs.

### REQ-E5NYR-004: Default gateway boundary

The caller MUST omit `gateway_url` and `oidc_audience` so the pinned Oidrune workflow uses its default gateway and audience. The caller MUST NOT pass the legacy `SHOUTRRR_URL` secret or any other Telegram secret.

### REQ-E5NYR-005: Complete failure summary

The failure path MUST pass `outcome: failure` and a caller-generated summary containing a failure title, project name, status/result, target SHA, run URL, workflow, ref, attempt, actor, and event. `Site Publish` failures MUST retain deployment-specific title and note semantics.

### REQ-E5NYR-006: Manual smoke path

The `workflow_dispatch` path MUST run only the smoke job and MUST pass `outcome: failure` with a caller-generated summary titled `IsolaRail notifier smoke test`. The summary MUST include the project name, smoke status/result, current SHA, run URL, workflow, ref, attempt, actor, and event.

### REQ-E5NYR-007: Contract coverage

The repository MUST provide a repeatable workflow contract test covering the pinned target, legacy target/secret removal, OIDC permission, summary fields, failure filter, ordinary PR exclusion, and manual smoke path.

## Verification

### VER-E5NYR-001: Static workflow contract

- Method: `actionlint` and the workflow contract test cover: `REQ-E5NYR-001`, `REQ-E5NYR-002`, `REQ-E5NYR-003`, `REQ-E5NYR-004`, `REQ-E5NYR-005`, `REQ-E5NYR-006`, `REQ-E5NYR-007`.

### VER-E5NYR-002: Documentation contract

- Method: `spec_contract_check.py`, `spec_drift_check.sh`, and markdownlint cover: `REQ-E5NYR-002`, `REQ-E5NYR-003`, `REQ-E5NYR-004`, `REQ-E5NYR-005`, `REQ-E5NYR-006`.

### VER-E5NYR-003: Repository validation

- Method: repository `cargo check` and `cargo build --release` cover: `REQ-E5NYR-001`, `REQ-E5NYR-007`.

### VER-E5NYR-004: Delivery safety

- Method: PR CI freshness and manual inspection confirm that no `workflow_dispatch` smoke run, real Telegram/Oidrune notification, or Oidrune control-plane change is performed during migration; covers: `REQ-E5NYR-004`, `REQ-E5NYR-006`.

## Related ADRs

None
