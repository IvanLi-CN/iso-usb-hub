import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const workflowPath = fileURLToPath(
  new URL("../.github/workflows/notify-release-failure.yml", import.meta.url),
);
const workflow = readFileSync(workflowPath, "utf8");
const oidruneTarget =
  "IvanLi-CN/oidrune/.github/workflows/notify.yml@e48822f99c6402a753ed86557ea029754cbab20b";
const oldTarget =
  "IvanLi-CN/github-workflows/.github/workflows/release-failure-telegram.yml";

function jobBlock(name) {
  const match = workflow.match(
    new RegExp(`\\n  ${name}:\\n([\\s\\S]*?)(?=\\n  [A-Za-z0-9_]+:\\n|$)`),
  );
  assert.ok(match, `missing job ${name}`);
  return match[1];
}

function assertSummaryContract(block, targetSha, result) {
  assert.match(block, /outcome: failure/);
  assert.match(block, /summary: \|/);
  assert.match(block, /title: /);
  assert.match(block, /project: \$\{\{ github\.repository \}\}/);
  assert.match(block, /status: /);
  assert.match(block, new RegExp(`result: ${result}`));
  assert.match(block, new RegExp(`target_sha: ${targetSha}`));
  assert.match(block, /run_url: /);
  assert.match(block, /workflow: /);
  assert.match(block, /ref: /);
  assert.match(block, /attempt: /);
  assert.match(block, /actor: /);
  assert.match(block, /event: /);
}

assert.equal(
  workflow.split(oidruneTarget).length - 1,
  2,
  "both notification jobs must use the pinned Oidrune workflow",
);
assert.ok(!workflow.includes(oldTarget), "the legacy Telegram workflow must be absent");
assert.ok(!workflow.includes("@main"), "reusable workflow references must be pinned");
assert.ok(!workflow.includes("SHOUTRRR_URL"), "the legacy Telegram secret must be absent");
assert.ok(!workflow.includes("gateway_url"), "gateway_url must use the Oidrune default");
assert.ok(!workflow.includes("oidc_audience"), "oidc_audience must use the Oidrune default");
assert.match(workflow, /(^|\n)permissions:\n  id-token: write\n/);
assert.match(workflow, /workflows:\n      - Release\n      - Development Release\n      - Site Publish/);
assert.match(workflow, /types:\n      - completed/);
assert.ok(!workflow.includes("\n  pull_request:\n"), "the notifier must not trigger on ordinary pull requests");
assert.match(
  workflow,
  /github\.event\.workflow_run\.name != 'Site Publish' \|\| github\.event\.workflow_run\.event != 'pull_request'/,
);

const failureJob = jobBlock("notify_failure");
assertSummaryContract(
  failureJob,
  "\\$\\{\\{ github\\.event\\.workflow_run\\.head_sha \\}\\}",
  "\\$\\{\\{ github\\.event\\.workflow_run\\.conclusion \\}\\}",
);
assert.match(failureJob, /github\.event\.workflow_run\.name == 'Site Publish'/);
assert.match(failureJob, /'IsolaRail Site Publish deployment failure'/);
assert.match(failureJob, /'IsolaRail release workflow failure'/);
assert.match(failureJob, /'Site Publish deployment failed'/);
assert.match(failureJob, /'Release workflow failed'/);

const smokeJob = jobBlock("smoke_test");
assert.match(smokeJob, /if: \$\{\{ github\.event_name == 'workflow_dispatch' \}\}/);
assertSummaryContract(smokeJob, "\\$\\{\\{ github\\.sha \\}\\}", "simulated-failure");
assert.match(smokeJob, /title: IsolaRail notifier smoke test/);
assert.match(smokeJob, /event: workflow_dispatch/);

console.log("notify-release-failure workflow contract: ok");
