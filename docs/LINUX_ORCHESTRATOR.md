# Linux Orchestrator Runbook

Read `docs/ORCHESTRATION.md` first. This runbook is for the Linux/Nix host orchestrator.

## Authority

The Linux orchestrator owns validation for:

- Nix shell and Linux CI parity.
- Rust host workspace behavior when it depends on Linux-only services.
- Hyprland remote output lifecycle.
- PipeWire, XDPH, DMA-BUF, format/modifier, and sync evidence.
- NVENC and NVIDIA driver behavior.
- Host daemon runtime behavior.
- Linux side of cross-device connectivity, streaming, and latency tests.

The Linux orchestrator may also assign Rust protocol, transport, telemetry, CLI, and testkit nodes to local or Mac
implementors when those nodes are platform-neutral.

## Startup

```sh
git pull --rebase
qd sync --from roadmap/qd-export.json --expect-clean
qd doctor --strict
nix develop -c just verify
qd ready --json
```

Post a coordination issue status comment after startup.

## qd Duties

The Linux orchestrator is responsible for keeping Linux-owned nodes honest in qd:

- Claim Linux host nodes before spawning implementors.
- Add blockers for unavailable host services, GPU state, compositor behavior, or driver issues.
- Require evidence for hardware claims before audit pass.
- Request Mac validation through qd/GitHub instead of marking Mac-dependent nodes complete locally.
- Export roadmap changes after adding or splitting Linux prerequisites.

Before CI promotion on Linux-owned nodes:

```sh
qd prompt audit <node-id>
qd audit start <node-id>
qd finding add ...
qd finding resolve ...
qd audit pass <node-id>
nix develop -c just verify
```

If qd has a configured gate command, run it before requesting CI or merge.

## Parallel Batch Shape

A good Linux batch has two to four implementors:

- One host/hardware node.
- One Rust shared-code node.
- One fixture or evidence tooling node.
- One audit/hardening node.

Example:

```text
m1-hyprland-fixtures
m1-compositor-contract
m1-hostd-control-cli
m0-ci-green
```

## Implementor Environment

Each Linux implementor should run inside Nix:

```sh
nix develop
qd worktree create <node-id>
```

Default checks:

```sh
nix develop -c just check
nix develop -c just test
```

For merge-ready host work:

```sh
nix develop -c just verify
nix develop -c just coverage
nix develop -c just security
```

## Evidence Standards

Linux evidence should include exact commands and environment:

```text
evidence/<node-id>/commands.log
evidence/<node-id>/host-env.txt
evidence/<node-id>/host.log
evidence/<node-id>/notes.md
```

Hardware nodes should also include whichever artifacts apply:

```text
hyprland-monitors.json
hyprland-workspaces.json
pipewire-nodes.json
capture-metadata.json
dmabuf-metadata.json
encode-session.json
sample.ivf
perfetto-trace.pftrace
screenshots/
```

Never claim "no CPU readback", "zero copy", HDR preservation, or latency targets without evidence.

## Cross-Device Duties

For cross-device nodes, Linux provides:

- Host command.
- Host IP/port or connection method.
- Host logs.
- Capture/encode/network timestamps.
- Firewall and service state notes.
- Artifact hashes for any sample streams.

Use `docs/CROSS_DEVICE_VALIDATION.md` for the live test protocol.

## GitHub Coordination

Ask the Mac orchestrator for validation with a coordination issue comment:

```md
## VALIDATION-REQUEST linux <timestamp>

Node: <node-id>
Branch/PR: <branch-or-url>
Mac command: <command>

Expected:
- <observable result>

Artifacts:
- <paths or PR artifacts>
```

Keep the PR blocked with `needs:mac-validation` until the Mac orchestrator posts evidence.

## Findings

Linux findings should be written when:

- tests pass but hardware evidence is missing
- a command works only in the developer session and not an isolated test context
- logs show compositor, PipeWire, DMA-BUF, or NVENC warnings
- acceptance criteria are ambiguous
- a PR changes shared protocol or telemetry without Mac compatibility evidence

Resolve findings with code, tests, evidence, or a documented node split. Do not resolve a hardware finding with a
unit test alone.

## When Linux Is Idle

If no Linux-only node is ready, spawn implementors on platform-neutral work:

- Protocol fixtures and golden vectors.
- Transport state machines.
- Telemetry schemas and report readers.
- qd/evidence tooling.
- Rust tests and property-test scaffolds.
- Documentation that unblocks both machines.
