# Orchestration Runbook

This runbook is for long-lived Codex orchestrator sessions. Read it before the platform-specific runbook.

madobe uses two persistent orchestrators:

- Linux/Nix orchestrator: the reference host, hardware, Nix, Rust, Hyprland, capture, NVENC, and host evidence.
- macOS orchestrator: the MacBook client, Xcode/Tuist, Swift, VideoToolbox, Metal, and Mac evidence.

Both orchestrators may develop Rust and repo-wide code. Platform ownership is about validation authority, not
implementation permission. macOS-only behavior must be validated on the Mac. Linux host behavior must be validated
on the Linux workstation.

The shared command surface is `just`. The environment boundary is platform-specific: the Linux orchestrator usually
runs those commands inside Nix, while the macOS orchestrator runs them with native Xcode, Tuist, Mise, and Homebrew
tools on PATH.

## Roles

Orchestrators coordinate. They keep qd synced, choose ready work, spawn implementors, audit results, adapt the DAG,
coordinate cross-device tests, and use GitHub for durable handoffs.

Implementors execute. Each implementor gets one qd node, one worktree, and one scoped prompt. Implementors should
not broaden scope or reshape the DAG directly. They report blockers and proposed splits back to the orchestrator.

## Shared State

Use these durable coordination surfaces:

- qd: dependency graph, claims, blockers, evidence state, audit state.
- `roadmap/qd-export.json`: portable qd state committed through Git.
- GitHub PRs: code review, CI, branch history, node-specific discussion.
- GitHub coordination issue: cross-machine status, validation requests, blocker handoffs, and decisions.
- `evidence/<node-id>/`: command logs, traces, screenshots, samples, validation outputs.
- `reports/qd/<node-id>/`: completion, audit, blocker, and DAG-change notes.

Do not use chat memory as the source of truth for work assignment.

## Coordination Issue

Create and pin one issue named:

```text
Orchestrator Coordination Log
```

Use structured comments:

```md
## ORCH-STATUS <linux|macos> <timestamp>

Active:
- <node-id> -> <branch-or-pr>

Implementors:
- <node-id> -> <agent-name> -> <state>

Blocked:
- <node-id>: <exact blocker>

Needs other platform:
- <node-id>: <request, branch, command, evidence wanted>

Decisions:
- <decision and rationale>
```

Post status at startup, before shutdown, after claiming multiple nodes, and whenever a blocker needs the other
platform.

## Labels

Use labels to make GitHub searchable:

```text
platform:linux
platform:macos
platform:both
role:orchestrator
role:implementor
needs:linux-validation
needs:mac-validation
needs:cross-device-validation
blocked
ready
audit-needed
evidence-needed
```

## Startup

Every orchestrator starts with:

```sh
git pull --rebase
qd sync --from roadmap/qd-export.json --expect-clean
qd doctor --strict
qd ready --json
```

Then read the latest coordination issue comments and open PRs with `qd/` branches.

## qd Lifecycle

Use qd as an evidence ledger, not only a task queue. The normal node lifecycle is:

1. Sync roadmap export.
2. Inspect ready queue.
3. Claim node.
4. Spawn implementor in a worktree.
5. Collect completion evidence.
6. Audit the node.
7. Resolve findings.
8. Run the qd gate and GitHub CI.
9. Merge through GitHub.
10. Record qd merge state and export the roadmap.

Useful commands:

```sh
qd status --json
qd ready --json
qd node show <node-id> --full
qd claim <node-id> --agent <agent-name>
qd worktree create <node-id>
qd export --out roadmap/qd-export.json --deterministic
```

If command names differ on the installed qdcli, run the relevant `qd <command> --help` and update this runbook in
the same PR.

## Work Selection

Prefer multiple independent nodes in parallel. Good batches have low file overlap and clear ownership:

- One platform validation node.
- One implementation node.
- One audit or hardening node.
- One fixture/tooling node.

Avoid assigning two implementors to nearby files unless the orchestrator is intentionally sequencing them.

## Implementor Spawn Contract

Prompt implementors like this:

```text
You are an implementor, not the orchestrator.
Complete qd node <node-id> only.
Use a qd worktree and branch qd/<node-id>-<short-topic>.
Do not edit qd graph topology. If the node is wrong, write reports/qd/<node-id>/dag-notes.md.
If blocked, write reports/qd/<node-id>/blocker.md with commands and logs.
Run the node acceptance checks and record evidence under evidence/<node-id>/.
```

Each implementor should produce:

```text
evidence/<node-id>/commands.log
evidence/<node-id>/notes.md
reports/qd/<node-id>/completion.json
```

The orchestrator audits the result before marking qd completion or requesting CI promotion.

## Dynamic DAG Changes

When work reveals new facts, the orchestrator decides whether to:

- Fix within the current node.
- Split a prerequisite node.
- Add a validation node.
- Move validation to the other platform.
- Block the node with an exact missing condition.
- Close the path and add a removal node.

Record DAG changes in qd, export deterministically, and send them through a small PR when possible:

```sh
qd export --out roadmap/qd-export.json --deterministic
```

Only one orchestrator should make topology changes at a time unless the edits are coordinated in the issue.

## Blockers

Block instead of weakening acceptance criteria. A blocker should record:

- node id
- exact command or operation that failed
- platform
- missing hardware, credential, dependency, API behavior, or remote validation
- evidence path
- proposed unblock path

Use qd's blocker/status mechanism when available, and always write:

```text
reports/qd/<node-id>/blocker.md
```

Post cross-machine blockers to the coordination issue with the `blocked` and platform labels on the PR or issue.

## PR Contract

Every PR title starts with the node id:

```text
<node-id>: <short imperative title>
```

Every PR body includes:

- qd node id
- acceptance criteria
- commands run
- evidence paths
- platform validation status
- cross-device validation status if relevant
- residual risks
- DAG changes proposed or made

## Audit

The orchestrator that did not spawn the implementor should audit when practical. For platform-specific work, audit
can happen on either machine, but final validation authority stays with the relevant platform.

Do not mark a node complete solely because tests pass. Completion requires the acceptance criteria and evidence.

Audit flow:

```sh
qd prompt audit <node-id>
qd audit start <node-id>
qd finding add ...
qd finding resolve ...
qd audit pass <node-id>
```

Audit must inspect:

- diff
- acceptance criteria
- completion report
- commands run
- artifacts
- hardware claims
- failure paths
- dependency or DAG changes

CI passing is evidence, not an audit. Findings should be concrete, reproducible, and either resolved or explicitly
accepted as residual risk before CI promotion.

## CI And Merge

GitHub is the merge authority. qd should not merge code by itself.

Use this sequence:

1. Complete node with evidence.
2. Audit node and resolve findings.
3. Run qd gate.
4. Open or update PR.
5. GitHub CI passes.
6. Required platform or cross-device validation passes.
7. Merge through GitHub merge queue.
8. Pull main locally.
9. Record qd merge state.
10. Export `roadmap/qd-export.json`.

The qd merge record happens after the real repository merge so qd remains aligned with main.

## Broad Reviews

Run broader qd review after major evidence changes, new hardware facts, or a large batch of merged nodes:

```sh
qd prompt dag-review
qd graph
qd critical-path
```

Trigger a DAG review when:

- a platform API behaves differently than expected
- validation is repeatedly flaky
- a dependency edge was discovered during implementation
- hardware evidence contradicts the plan
- too many ready nodes touch the same ownership boundary
