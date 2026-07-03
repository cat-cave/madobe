# qd Orchestration Smoke Evidence

Node: `m0-qd-orchestration-smoke`
Date: 2026-07-03
Validator: `trevor-linux-orchestrator`

## Result

Passed.

## qd Outputs

`qd status --json` after the smoke:

```json
{
  "nodes": 25,
  "ready": 2,
  "byStatus": {
    "done": 7,
    "claimed": 3,
    "ready": 15
  },
  "donePoints": 7,
  "totalPoints": 25,
  "remainingPoints": 18,
  "openP0P1Findings": 0
}
```

`qd ready --json` showed the remaining ready queue after Linux claimed
`m0-qd-orchestration-smoke`, `m1-hyprland-command-layer`, and `m2-capture-contract`:

```text
m3-swift-protocol-mirror
m3-quic-transport-skeleton
```

## Coordination Issue

Durable coordination bus:

- [Orchestrator Coordination Log](https://github.com/cat-cave/madobe/issues/1)

Relevant structured comments:

- [Linux initial wave](https://github.com/cat-cave/madobe/issues/1#issuecomment-4877897814)
- [Mac protocol claim/status](https://github.com/cat-cave/madobe/issues/1#issuecomment-4877668272)
- [Mac protocol PR and Linux validation request](https://github.com/cat-cave/madobe/issues/1#issuecomment-4877763495)
- [Linux protocol merge/qd state update](https://github.com/cat-cave/madobe/issues/1#issuecomment-4878036393)
- [Mac protocol completion update](https://github.com/cat-cave/madobe/issues/1#issuecomment-4878040145)
- [Linux second wave](https://github.com/cat-cave/madobe/issues/1#issuecomment-4878045297)

## Claimed Nodes

Mac-owned/shared claim observed:

- `m3-video-protocol` was claimed and completed by `trevor-mac-orchestrator`, then merged through PR #7.

Linux-owned/shared claims observed:

- `m1-hyprland-fixtures` and `m1-compositor-contract` were claimed, implemented, audited, merged, and recorded done.
- `m1-hyprland-command-layer` is claimed by `trevor-linux-orchestrator`.
- `m2-capture-contract` is claimed by `trevor-linux-orchestrator`.

## Git Hygiene

`git ls-files .qd` output:

```text
.qd/agents.md
.qd/config.toml
```

No `.qd/qd.db` file is tracked.

Current main CI evidence:

- [qd merge state retry green run](https://github.com/cat-cave/madobe/actions/runs/28672673486)
