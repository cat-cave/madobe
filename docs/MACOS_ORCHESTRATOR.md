# macOS Orchestrator Runbook

Read `docs/ORCHESTRATION.md` first. This runbook is for the MacBook orchestrator.

## Authority

The macOS orchestrator owns validation for:

- Tuist generation.
- Xcode build and test.
- SwiftFormat and SwiftLint behavior on macOS.
- Swift client core behavior.
- VideoToolbox decode behavior.
- Metal rendering behavior.
- macOS app shell behavior.
- Mac side of cross-device connectivity, streaming, and latency tests.

The Mac orchestrator may also develop Rust protocol, transport, telemetry, CLI, and testkit code. Only validation
that requires Linux host services should be handed back to the Linux orchestrator.

## Startup

```sh
git pull --rebase
qd sync --from roadmap/qd-export.json --expect-clean
qd doctor --strict
nix develop -c just check
nix develop -c just apple-generate
nix develop -c just apple-test
qd ready --json
```

Post a coordination issue status comment after startup.

## qd Duties

The Mac orchestrator is responsible for keeping Mac-owned nodes honest in qd:

- Claim Mac validation and client nodes before spawning implementors.
- Add blockers for missing Xcode, Tuist, signing, simulator/device, VideoToolbox, or Metal conditions.
- Require real Xcode or macOS command evidence before audit pass.
- Request Linux validation through qd/GitHub instead of marking host-dependent nodes complete locally.
- Export roadmap changes after adding or splitting Mac prerequisites.

Before CI promotion on Mac-owned nodes:

```sh
qd prompt audit <node-id>
qd audit start <node-id>
qd finding add ...
qd finding resolve ...
qd audit pass <node-id>
nix develop -c just apple-test
```

If qd has a configured gate command, run it before requesting CI or merge.

## Parallel Batch Shape

A good Mac batch has two to four implementors:

- One macOS validation or app node.
- One Swift client node.
- One platform-neutral Rust node.
- One fixture or evidence-reader node.

Example:

```text
m0-mac-verify
m3-swift-protocol-mirror
m3-videotoolbox-decode-sample
m3-latency-report-viewer
```

If there is no Mac-specific work ready, keep the Mac busy with platform-neutral Rust work. Prefer shared protocol,
transport, telemetry, and testkit nodes that do not need Linux hardware validation.

## Implementor Environment

Each Mac implementor should use a qd worktree:

```sh
qd worktree create <node-id>
```

Default checks:

```sh
nix develop -c just check
nix develop -c just apple-test
```

For Rust-only nodes on Mac:

```sh
nix develop -c just check
nix develop -c just test
```

For client nodes, also run the relevant Xcode command and record the destination:

```sh
xcodebuild test -scheme MadobeMac -destination 'platform=macOS'
```

## Evidence Standards

Mac evidence should include:

```text
evidence/<node-id>/commands.log
evidence/<node-id>/mac-env.txt
evidence/<node-id>/xcodebuild.log
evidence/<node-id>/client.log
evidence/<node-id>/notes.md
```

Video and renderer nodes should also include:

```text
decode-report.json
render-timeline.json
sample-input-hash.txt
screenshots/
```

## Cross-Device Duties

For cross-device nodes, macOS provides:

- Client command.
- Client logs.
- Decode/render timestamps.
- VideoToolbox capability notes.
- Screenshots or recordings when visually relevant.
- Network observations from the client side.

Use `docs/CROSS_DEVICE_VALIDATION.md` for live tests.

## GitHub Coordination

Ask Linux for host validation with a coordination issue comment:

```md
## VALIDATION-REQUEST macos <timestamp>

Node: <node-id>
Branch/PR: <branch-or-url>
Linux command: <command>

Expected:
- <observable result>

Artifacts:
- <paths or PR artifacts>
```

Keep the PR blocked with `needs:linux-validation` until the Linux orchestrator posts evidence.

## Findings

Mac findings should be written when:

- Tuist generation is unverified
- Xcode succeeds locally but the command is not recorded
- VideoToolbox behavior differs from assumptions
- Metal presentation works visually but lacks timing evidence
- Swift mirrors Rust protocol data without golden-vector proof
- a PR changes shared protocol or telemetry without Linux compatibility evidence

Resolve findings with code, tests, evidence, or a documented node split. Do not resolve a Mac validation finding
with Linux-only checks.

## When macOS Is Idle

If no Mac-only node is ready, spawn implementors on:

- Rust protocol and golden-vector work.
- QUIC/session state machines.
- Telemetry schemas.
- CLI commands that do not require host services.
- Evidence parsers and report rendering.
- Documentation and runbook updates.
