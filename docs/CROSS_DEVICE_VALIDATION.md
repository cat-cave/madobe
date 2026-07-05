# Cross-Device Validation Runbook

Use this runbook when a node needs both the Linux host and the MacBook client.

Cross-device validation is a coordinated activity between the two orchestrators. Implementors may prepare commands,
fixtures, and tools, but orchestrators run or supervise the live test and record the final evidence.

## Validation Node Shape

Create explicit validation nodes for real connectivity or media claims:

```text
<milestone>-cross-device-<capability>-smoke
<milestone>-cross-device-<capability>-latency
<milestone>-cross-device-<capability>-soak
```

Examples:

```text
m3-cross-device-video-smoke
m3-cross-device-frame-latency
m4-cross-device-input-roundtrip
```

## Preflight

Both orchestrators sync to the same branch or PR:

```sh
git fetch
git switch <branch>
git pull --ff-only
qd sync --from roadmap/qd-export.json --expect-clean
```

Record environment:

```sh
mkdir -p evidence/<node-id>
uname -a > evidence/<node-id>/<platform>-uname.txt
```

Platform-specific preflight should record:

- Linux: GPU, driver, Hyprland version, PipeWire version, host IP, firewall state.
- macOS: hardware model, macOS version, Xcode version, client IP, network interface.

## Live Test Handshake

Use the coordination issue or PR comment for the live handshake:

```md
## LIVE-TEST <node-id> <timestamp>

Branch:
Linux ready:
- command:
- listening address:
- evidence dir:

Mac ready:
- command:
- evidence dir:

Start order:
1. Linux host
2. Mac client
3. Linux stop after Mac confirms result
```

The orchestrator that owns the validation node updates the comment as the test progresses.

For product QUIC smoke nodes, use a receiver-first handshake because the Mac receiver creates the pinned
certificate before the Linux sender can connect:

```md
## PRODUCT-QUIC-READY <node-id> <timestamp>

Branch/PR:
Commit:

Mac receiver:
- bind:
- LAN address:
- UDP port/firewall:
- command:
- evidence dir: evidence/<node-id>/macos-receiver/
- server cert: evidence/<node-id>/macos-receiver/server-cert.der
- server cert sha256:
- ready line:

Linux sender:
- command:
- evidence dir: evidence/<node-id>/linux-sender/
- expected success line:

Validation boundary:
- claims: product QUIC transport, byte count, SHA-256, receiver ack
- non-claims: VideoToolbox decode, Metal render, presentation, latency
```

Use this command shape on the product QUIC branch or merged commit:

```sh
madobectl product-quic-smoke receive \
  --bind <mac-lan-ip:udp-port> \
  --cert-san <mac-lan-ip> \
  --evidence-dir evidence/<node-id>/macos-receiver

madobectl product-quic-smoke send \
  --addr <mac-lan-ip:udp-port> \
  --server-name <mac-lan-ip> \
  --server-cert-der evidence/<node-id>/macos-receiver/server-cert.der \
  --evidence-dir evidence/<node-id>/linux-sender
```

If the certificate has to cross machines through GitHub, commit or attach only the node-scoped
`server-cert.der` and its SHA-256. Do not reuse it for later runs.

## Required Evidence

Every cross-device validation produces:

```text
evidence/<node-id>/commands.log
evidence/<node-id>/linux-host.log
evidence/<node-id>/mac-client.log
evidence/<node-id>/result.json
evidence/<node-id>/notes.md
```

Metrics-focused tests also produce:

```text
frame-timeline.json
network-timeline.json
latency-summary.json
perfetto-trace.pftrace
```

Streaming tests should include a sample or hash:

```text
sample.ivf
sample.sha256
```

Visual tests should include screenshots:

```text
screenshots/linux/
screenshots/macos/
```

Product QUIC smoke tests use platform subdirectories:

```text
evidence/<node-id>/macos-receiver/server-cert.der
evidence/<node-id>/macos-receiver/server-cert.sha256
evidence/<node-id>/macos-receiver/receiver-listening.log
evidence/<node-id>/macos-receiver/receiver.log
evidence/<node-id>/macos-receiver/receiver-timeline.json
evidence/<node-id>/macos-receiver/result.json
evidence/<node-id>/linux-sender/sender.log
evidence/<node-id>/linux-sender/sender-timeline.json
evidence/<node-id>/linux-sender/network-notes.md
```

The receiver log must show `transport=quic`, `product_quic=true`, payload byte-count validation, payload SHA-256
validation, and the explicit decode/render/presentation/latency non-claim. Sender evidence must include the
trusted certificate path or hash and the typed transport error if the run fails before the receiver ack.

## Result Schema

Use this shape for `result.json`:

```json
{
  "node_id": "m3-cross-device-video-smoke",
  "branch": "qd/m3-cross-device-video-smoke",
  "linux_commit": "",
  "macos_commit": "",
  "started_at": "",
  "ended_at": "",
  "passed": false,
  "metrics": {
    "frames_sent": 0,
    "frames_decoded": 0,
    "frames_rendered": 0,
    "frames_presented": 0,
    "median_glass_to_glass_ms": null,
    "p95_glass_to_glass_ms": null
  },
  "artifacts": [
    {
      "path": "evidence/m3-cross-device-video-smoke/commands.log",
      "kind": "commands_log"
    }
  ],
  "notes": ""
}
```

Null is acceptable for latency metrics the current milestone cannot measure yet. Do not invent numbers.

Artifact `kind` values that support metric claims are:

- `decode_evidence` for nonzero `frames_decoded`
- `render_evidence` for nonzero `frames_rendered`
- `presentation_evidence` for nonzero `frames_presented`
- `latency_evidence` for non-null `median_glass_to_glass_ms` or `p95_glass_to_glass_ms`

Other useful kinds are `commands_log`, `linux_host_log`, `mac_client_log`, `notes`, and `other`. A checked schema
fixture lives at `crates/protocol/fixtures/m3-cross-device-video-smoke/result.json`; it records only schema
compatibility and does not claim live cross-device decode, render, presentation, or latency behavior.

For product QUIC smoke, use the checked Rust contract and golden fixture:

- model: `crates/protocol/src/product_quic.rs`
- fixture: `crates/protocol/fixtures/m4-product-quic-smoke/result.json`

The product QUIC result contract requires `transport=quic`, `productQuic=true`, sender and receiver endpoint
roles, payload byte count, SHA-256 payload validation, receiver acknowledgement, and optional certificate
fingerprint SHA-256. Downstream decode, render, presentation, and latency fields are explicit non-claims unless
the result also includes matching downstream evidence artifacts.

The golden fixture uses this shape:

```json
{
  "nodeId": "m4-product-quic-cross-device-smoke",
  "branch": "spec/m4-product-quic-result-schema",
  "transport": "quic",
  "productQuic": true,
  "sender": {
    "role": "sender",
    "platform": "linux",
    "evidenceDir": "evidence/m4-product-quic-cross-device-smoke/linux-sender"
  },
  "receiver": {
    "role": "receiver",
    "platform": "macos",
    "evidenceDir": "evidence/m4-product-quic-cross-device-smoke/macos-receiver"
  },
  "payload": {
    "payloadBytes": 84,
    "sha256": "3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a",
    "byteCountValidated": true,
    "sha256Validated": true
  },
  "receiverAck": {
    "received": true,
    "payloadBytes": 84,
    "sha256": "3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a"
  },
  "certificateFingerprintSha256": "9b44d90fb42f6c3ff8510ce40bbfcb1cf8712a2d18a3552955aa1b889ad2c6f3",
  "downstreamClaims": {
    "decoded": false,
    "rendered": false,
    "presented": false,
    "latencyMs": null
  },
  "artifacts": [
    {
      "path": "evidence/m4-product-quic-cross-device-smoke/commands.log",
      "kind": "commands_log"
    }
  ],
  "notes": "Contract fixture only; no live product QUIC, cross-device, decode, render, presentation, or latency behavior is claimed."
}
```

The product QUIC result proves transport and payload validation only. Downstream decode, render, presentation, and
latency nodes need their own evidence.

`sender.evidenceDir`, `receiver.evidenceDir`, and every artifact `path` must be repo-relative forward-slash
references. Absolute paths, backslashes, Windows drive prefixes, and `..` traversal components are rejected; artifact
`kind` must use the vocabulary modeled in `crates/protocol/src/product_quic.rs`.

Validate product QUIC result artifacts before using them as qd evidence:

```sh
just product-quic-result-check evidence/m4-product-quic-cross-device-smoke/macos-receiver/result.json
```

With no arguments, the same command validates the checked-in golden fixture and negative fixtures for obsolete
flat schema fields and unsupported downstream claims. `just check` runs that default validation in local and CI
quality gates.

## Failure Handling

If validation fails, do not hide it in local logs. Record:

- exact failing command
- host/client logs
- network state
- whether the failure is deterministic
- proposed new node or fix

Then the owning orchestrator chooses:

- fix current node
- split Linux prerequisite
- split macOS prerequisite
- add instrumentation node
- block on unavailable hardware or credentials

Create qd findings for failed or incomplete validation:

```sh
qd finding add ...
qd finding resolve ...
```

Findings are required for:

- missing Linux evidence
- missing Mac evidence
- mismatched commits between host and client
- unmeasured latency claims
- dropped frames without analysis
- decode/render success without visible or logged proof
- connectivity failures that require a new prerequisite node

## Completion Rule

A cross-device node is complete only when both orchestrators have posted evidence or one orchestrator explicitly
records why the other platform was not required for that node.

Before completion:

```sh
qd prompt audit <node-id>
qd audit start <node-id>
qd audit pass <node-id>
```

After GitHub merge, the owning orchestrator records qd merge state and exports `roadmap/qd-export.json`.
