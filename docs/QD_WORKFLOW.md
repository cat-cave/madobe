# madobe qd Workflow

Status: initial operating model, July 2, 2026

madobe should use qd as an evidence-first roadmap ledger, not as a lightweight task list. The goal is to let work happen in parallel from the Linux workstation, the MacBook, and a collaborator's machine without duplicate effort, hidden blockers, or optimistic "done" states.

GitHub remains the merge authority. qd owns the dependency graph, claims, evidence, audits, and merge recording after the real PR has landed.

## Source of Truth

Commit these files:

- `.qd/config.toml`: shared qd policy and command configuration.
- `.qd/agents.md`: shared agent/contributor conventions.
- `roadmap/qd-export.json`: deterministic qd graph export.
- `docs/QD_WORKFLOW.md`: human operating rules.

Do not commit qd's local SQLite state:

- `.qd/qd.db`
- `.qd/qd.db-*`
- `.qd/worktrees/`
- `.qd/skills/`

The local database is a working cache. The committed export is the portable roadmap artifact. This avoids binary merge conflicts when the Linux workstation, MacBook, and coworker all advance local qd state.

Recommended sync loop:

```sh
git pull --rebase
qd sync --from roadmap/qd-export.json --expect-clean
qd ready --json
```

After changing qd graph state:

```sh
qd export --out roadmap/qd-export.json --deterministic
git add roadmap/qd-export.json
```

When exporting qd state for a qd worktree branch, run `qd export` from the main checkout or qd root that has the qd
database and migrations, and write the result into the worktree path:

```sh
qd export --out .qd/worktrees/<node-id>/roadmap/qd-export.json --deterministic
```

Do not run `qd export` from a qd worktree that lacks the qd database or migrations.

If two branches modify the roadmap, resolve the JSON as source, then run:

```sh
qd sync --from roadmap/qd-export.json --expect-clean --write-diff /tmp/qd-sync-diff.json
qd graph
```

## Method Gate

qd blocks mutating commands until the strict method is acknowledged locally. Each machine or agent should do this once:

```sh
qd method show
qd method acknowledge --agent trevor-linux
qd method acknowledge --agent trevor-mac
qd method acknowledge --agent collaborator
```

The method matters for madobe because most work touches real graphics, media, input, or transport surfaces. Research happens before nodes are created. Once work enters qd, it should be action with acceptance criteria: implement, validate, wire, harden, audit, or remove. Unknown APIs, unstable compositor behavior, unavailable hardware, missing credentials, and untested environments become typed blockers or pre-DAG research notes; they do not become vague qd nodes.

## qd Configuration Plan

Current config is intentionally conservative: merge requires audits, verification, CI, and clean worktrees. The qd
check command should stay platform-neutral and call the shared repo command surface.

Once the monorepo skeleton exists, set:

```sh
qd config set check-command "just check"
qd config set ci-command "gh workflow run ci.yml"
qd config set ci-provider github --repo <owner>/madobe --workflow ci.yml --auth gh-cli
qd doctor --strict
```

On Linux, run qd checks from inside the Nix environment when Nix-provided tools are needed. On macOS, run qd checks
after `just macos-bootstrap`, using the native Xcode/Tuist/Mise/Homebrew environment. Do not put a Nix command in
the shared qd check command, because that makes macOS orchestrators unable to use the same qd gate.

If qd's installed command names differ for config keys, update this document after verifying with `qd config --help`. Do not hand-edit policy around a failing doctor result unless the failure is understood.

## Node Model

Every qd node must be independently mergeable and evidence-backed. Nodes should be small enough that one contributor can finish, verify, audit, and merge them without bundling unrelated product decisions.

Use this split:

- Implementation nodes: build a known design with explicit acceptance criteria.
- Validation nodes: run hardware, latency, pixel, audio, input, soak, or interoperability checks.
- Audit nodes or audit records: independently review evidence, failure paths, and acceptance criteria.
- Removal nodes: delete a rejected path, obsolete adapter, compatibility shim, or diagnostic-only experiment before it becomes product surface.

Avoid "figure out X while implementing" nodes. For madobe, that wording means the DAG is not ready yet. Do the research outside qd, decide the action, then add nodes that can actually be completed.

## Initial Milestone Shape

Milestones should describe externally meaningful capability, not arbitrary calendar phases.

`M0: bootstrap the repo machine`

- Monorepo scaffold, Nix shell, Rust workspace, Swift package/app skeleton.
- qd export workflow committed and documented.
- GitHub PR checks and merge queue configured.
- No product feature code without lint, formatting, and dependency policy.
- This milestone is allowed to be awkward and partially manual because it exists to make qd practical for all later work: worktrees, checks, CI, and merge recording need somewhere real to run.

`M1: remote display ownership`

- Compositor adapter traits.
- Hyprland adapter creates, configures, parks, removes, and reconciles named remote outputs.
- Workspace/session binding through the adapter contract.
- Restart reconciliation and typed recovery behavior.
- No capture, encode, transport, or session code calls Hyprland directly.

`M2: GPU video path`

- Capture a Hyprland remote output through PipeWire/XDPH or a direct capture path.
- Prove DMA-BUF metadata, format/modifier negotiation, sync mode, and no CPU readback after capture.
- Encode AV1 through NVENC on the reference host.
- Persist artifacts: metadata dump, encoded sample, trace, and command logs.

`M3: Mac video endpoint`

- QUIC video lane carries encoded frames.
- macOS client decodes AV1 with VideoToolbox.
- Metal renderer presents frames with timing instrumentation.
- Frame pacing, loss handling, and decode/render latency have artifacts.

`M4: interactive remote session`

- macOS client connects over QUIC.
- Host creates or leases a remote output.
- AV1 video reaches VideoToolbox and Metal.
- Keyboard and pointer return to the host.
- Telemetry records capture, encode, network, decode, and render timings.

`M5: private daily desktop`

- Audio, reconnect, permissions, text clipboard, dynamic resize, stats overlay, and debug bundles.
- Hardware smoke validation on Linux host and Mac client.
- GitHub merge queue blocks product-impacting changes without relevant evidence.

`M6: KDE host parity`

- KWin backend satisfies the same compositor adapter contract for the collaborator's KDE stack.
- KWin virtual output ownership, stream lifecycle, workspace/window routing, and EIS/uinput input path are implemented or patched.
- KWin backend has its own hardware validation and does not weaken the Hyprland reference path.
- Any KDE-private protocol dependency is isolated in `compositor-kwin` and tracked with an upstream/contribute/fork decision.

`M7: game device layer`

- Relative mouse path is robust enough for games.
- Controller support includes hotplug, disconnect cleanup, and a strict supported profile set.
- Haptics is added only if it fits the input/device contract cleanly.
- Input latency and stuck-input failure paths are validated on hardware.

`M8: file and workflow layer`

- File transfer is resumable and permissioned.
- Clipboard supports the selected MIME set with direction controls.
- App/workspace launcher controls remote sessions without becoming a generic admin tool.
- Debug/admin actions are explicit, logged, and scoped to paired devices.

`M9: high-fidelity display`

- HDR/10-bit path is implemented where the Linux capture, NVENC, protocol metadata, VideoToolbox, and Metal/EDR path can preserve the signal.
- HEVC Main10 exists only if it beats AV1 for a concrete display/color use case.
- Multi-output sessions are implemented through the same display/session model, not as special cases.
- 1440p120, 4K60, and 4K120 targets become ratcheted hardware validations once achieved.

`M10: mobile Apple endpoint`

- iPadOS client shares protocol/session code where useful.
- Touch, pencil, software keyboard, controller, display scaling, and backgrounding constraints are handled as product decisions.
- iPadOS support does not change the host reference contract.

Do not add milestones named after uncertainty. If a topic is not actionable yet, keep it out of qd until the research produces implementation or validation work.

## Parallel Contributor Flow

All coding starts from the ready queue:

```sh
git pull --rebase
qd sync --from roadmap/qd-export.json --expect-clean
qd ready --json
qd node show <node-id> --full
```

Claim before starting work:

```sh
qd claim <node-id> --agent trevor-linux
```

Use a branch name that carries the qd node:

```text
qd/<node-id>-short-topic
```

For risky or concurrent work, use qd worktrees:

```sh
qd worktree create <node-id>
```

PR titles should start with the node id:

```text
<node-id>: implement Hyprland remote output lifecycle
```

PR descriptions should include:

- qd node id
- acceptance criteria
- evidence paths or artifact links
- qd completion report
- residual risks

No contributor should pick unclaimed work from memory or chat. If new work appears during implementation, add or propose a qd node and edge instead of silently expanding scope.

## Completion Evidence

Completion means ready for audit, not "probably done." qd's completion schema expects:

- summary
- changed files
- commits
- acceptance evidence
- commands run
- artifacts
- real-world validation
- unverified items
- DAG changes needed

For madobe, evidence should live under predictable paths:

```text
evidence/<node-id>/
  commands.log
  host.log
  client.log
  frame-timeline.json
  capture-metadata.json
  perfetto-trace.pftrace
  sample.ivf
  screenshots/
  notes.md

reports/qd/<node-id>/
  completion.json
  audit.json
```

Do not manufacture evidence for unavailable hardware. If the node depends on hardware, compositor, driver, network, credential, or macOS access that is unavailable, block the node with the correct typed blocker and record the exact missing condition.

Before using `qd complete --from-report`, make the completion report final enough for qd to accept it:

- `unverifiedItems` must be empty. Unresolved required validation should become a blocker, a split follow-up node, or
  stay out of `qd complete` until it is resolved.
- `realWorldValidation.status` must be `passed` or `not_required`.
- If hosted CI is the required real-world validation, record the pass with `qd ci record-pass` before merge and cite
  the hosted CI run in qd CI evidence and PR logs.

Do not use a completion report to hide deferred acceptance criteria. If evidence is missing, keep the node incomplete
or blocked instead of marking an item unverified and attempting completion.

## Audit Flow

Every meaningful node gets independent review before CI promotion:

```sh
qd prompt audit <node-id>
qd audit start <node-id>
qd finding add ...
qd finding resolve ...
qd audit pass <node-id>
```

Audit checks the diff, completion report, acceptance criteria, artifacts, failure paths, and claims about real hardware behavior. CI passing is useful evidence, but it is not an audit.

Audit and finding disposition happen before CI promotion. Resolve, accept, or split findings before treating hosted CI
as promotion evidence. Hosted CI logs belong in qd CI evidence and the PR before merge, not as a substitute for audit.

Use broader DAG review after reality changes or after roughly 30 merged nodes:

```sh
qd prompt dag-review
qd graph
qd critical-path
```

Examples of reality changes:

- Hyprland changes output lifecycle behavior.
- PipeWire/XDPH cannot supply stable DMA-BUF capture for named outputs.
- NVENC AV1 latency or VideoToolbox AV1 behavior differs from assumptions.
- KWin backend work requires an upstream KDE change, narrow fork, or home-rolled control surface.
- CI hardware proves a path too slow or too flaky.

## CI and Merge Queue

The GitHub branch protection model should be:

- PR fast gate required.
- qd gate required once configured.
- Merge queue required for `main`.
- Hardware smoke required only for changes touching product-critical paths.
- Nightly hardware soak is informative unless it exposes a release-blocking regression.

qd should not merge code by itself. The sequence is:

1. Complete node with evidence.
2. Audit node.
3. Run qd gate.
4. Open PR.
5. GitHub CI and required hardware checks pass.
6. Merge through GitHub merge queue.
7. Pull main locally.
8. Record qd merge state and export roadmap.

The qd merge record must happen after the real repository merge, because qd's method treats main staying green as non-negotiable.

## Hardware Reality

Use synthetic checks for protocol logic, state machines, serialization, permission policy, and scheduling. Use hardware checks for claims about capture, encode, input, decode, render, latency, and color.

Recommended runner roles:

- `linux-host-reference`: NixOS, Hyprland, Linux 7.x+, NVIDIA RTX 4090.
- `linux-host-collab`: Kubuntu/Ubuntu when available, modern kernel, NVIDIA RTX 5060 Ti-class GPU.
- `mac-client-reference`: M5 Pro MacBook, current macOS/Xcode.

Hardware jobs should be targeted by changed paths. A protocol-only PR should not wait for a full capture/encode/client run. A capture, encoder, compositor, input, or macOS renderer PR should produce hardware evidence.

## Duplicate Effort Controls

Daily or pre-session routine:

```sh
git pull --rebase
qd sync --from roadmap/qd-export.json --expect-clean
qd status --json
qd ready --json
qd critical-path
```

Rules:

- Claim before coding.
- One node, one branch, one PR unless explicitly split.
- Split nodes when acceptance criteria span unrelated hardware surfaces.
- Add edges when a hidden dependency appears.
- Block rather than weakening acceptance criteria.
- Export roadmap changes in the same PR that changes qd state.
- Do not let chat, local notes, or GitHub issue comments become the authoritative roadmap.
