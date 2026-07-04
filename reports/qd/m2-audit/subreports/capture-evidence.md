# M2 Capture Evidence Audit

Node: `m2-audit`

Scope: capture-related evidence and blockers for deciding whether Linux video samples are reliable M3 inputs.

## Reviewed files and evidence

- `docs/ORCHESTRATION.md`, `docs/LINUX_ORCHESTRATOR.md`, and `docs/CROSS_DEVICE_VALIDATION.md`.
- `roadmap/qd-export.json` entry for `m2-audit` because `qd node show m2-audit --full` was blocked by the local qd DB schema.
- `reports/qd/m2-pipewire-probe/{completion.json,audit.json,blocker.md}` and `evidence/m2-pipewire-probe/*` summaries.
- `reports/qd/m2-capture-stream-proof/{completion.json,audit.json,blocker.md}` and `evidence/m2-capture-stream-proof/*` summaries.
- `reports/qd/m2-portal-screencast-client-proof/{completion.json,audit.json,blocker.md}` and `evidence/m2-portal-screencast-client-proof/*` summaries.
- `reports/qd/m2-portal-chooser-interaction-proof/{completion.json,audit.json,blocker.md}` and `evidence/m2-portal-chooser-interaction-proof/*` summaries.
- `reports/qd/m2-portal-manual-consent-frame-proof/{completion.json,audit.json}` plus `evidence/m2-portal-manual-consent-frame-proof/{validation-summary.json,notes.md,portal-client.txt,pipewire-during-open-remote-summary.json}`.
- `reports/qd/m2-direct-capture-api-spike/{completion.json,audit.json}` plus `evidence/m2-direct-capture-api-spike/{validation-summary.json,notes.md,tools-and-protocols.md}`.
- `reports/qd/m2-native-dmabuf-allocation-proof/{completion.json,audit.json}` plus `evidence/m2-native-dmabuf-allocation-proof/{validation-summary.json,notes.md,native-dmabuf-capture.json}`.
- `reports/qd/m2-capture-contract/{completion.json,audit.json}` and `evidence/m2-capture-contract/notes.md`.
- `reports/qd/m2-capture-one-frame/{completion.json,audit.json,blocker.md}` plus `evidence/m2-capture-one-frame/{direct-capture-frame.json,validation-summary.json,notes.md}`.
- `reports/qd/m2-nvenc-encode-sample/{completion.json,audit.json}` plus `evidence/m2-nvenc-encode-sample/{notes.md,validation-summary.json,ffprobe-sample-av1.txt,sample-av1-header.txt,artifact-sha256.txt}`.
- `reports/qd/m2-video-path-validation/{completion.json,audit.json}` plus `evidence/m2-video-path-validation/{bundle-manifest.json,notes.md}`.
- Follow-up evidence-boundary reports under `reports/qd/m2-video-path-validation-does-not-*` and `evidence/m2-video-path-validation-does-not-*`.

## Findings by severity

### P0

None.

### P1

None for the capture evidence itself. The current direct capture evidence is sufficient to prove one Linux frame capture from a node-scoped Hyprland output via `ext-image-copy-capture-v1` using an NVIDIA GBM DMA-BUF, with size, format, modifier, timestamp, damage, and implicit/protocol-ready synchronization:

- `evidence/m2-capture-one-frame/direct-capture-frame.json:3` records `success=true`.
- `evidence/m2-capture-one-frame/direct-capture-frame.json:6` records constraints, advertised `/dev/dri/renderD128`, `AR24`, and NVIDIA block-linear modifier `0x0300000000606010`.
- `evidence/m2-capture-one-frame/direct-capture-frame.json:7` records GBM allocation on `/dev/dri/renderD128`, one plane, stride `7680`, and the same submitted modifier.
- `evidence/m2-capture-one-frame/direct-capture-frame.json:9` records `frame.ready=true`, `frame.failed=false`, `CLOCK_MONOTONIC` timestamp, full damage, and `implicit/protocol-ready` sync.
- `evidence/m2-capture-one-frame/validation-summary.json:16` through `evidence/m2-capture-one-frame/validation-summary.json:48` summarize the same required metadata.

### P2

- Stale contradictory blocker file remains in the completed one-frame node. `reports/qd/m2-capture-one-frame/blocker.md:3` says the direct Wayland probe did not complete a captured frame, and `reports/qd/m2-capture-one-frame/blocker.md:14` says the missing condition is `ext_image_copy_capture_frame_v1.ready`. That contradicts `reports/qd/m2-capture-one-frame/completion.json:3`, `reports/qd/m2-capture-one-frame/audit.json:10` through `reports/qd/m2-capture-one-frame/audit.json:18`, and the current direct capture JSON above. Disposition: treat the blocker as stale evidence residue, not as the current state, but remove or supersede it in a follow-up so future audits do not read a false blocker.

- The direct capture helper is evidence code, not a committed production capture implementation. `evidence/m2-native-dmabuf-allocation-proof/notes.md:92` through `evidence/m2-native-dmabuf-allocation-proof/notes.md:108` say the proof used a temporary `nix-shell` and the repo dev shell lacks native Wayland/GBM build dependencies. `reports/qd/m2-native-dmabuf-allocation-proof/audit.json:52` through `reports/qd/m2-native-dmabuf-allocation-proof/audit.json:55` record this residual risk. Disposition: acceptable for M2 evidence, but M3 should not assume a reusable repo capture helper exists until a follow-up adds the native capture helper/dependencies.

### P3

- Portal/PipeWire path remains manually consented and incomplete as an unattended product path. `reports/qd/m2-portal-manual-consent-frame-proof/completion.json:3` proves chooser approval, stream id, `OpenPipeWireRemote`, and PipeWire format/modifier metadata, but `reports/qd/m2-portal-manual-consent-frame-proof/completion.json:85` through `reports/qd/m2-portal-manual-consent-frame-proof/completion.json:88` state no frame was consumed and timestamp/sync still needed a frame consumer. The direct capture path now covers one-frame metadata, so this is not a blocker for narrow M3 sample inputs. It remains a product-path follow-up if portal reconnect/unattended capture is required.

- Linux AV1 sample is intentionally narrow. `evidence/m2-video-path-validation/notes.md:14` through `evidence/m2-video-path-validation/notes.md:26` identify `sample-av1.ivf` as a one-frame AV1 Main IVF artifact that is stable for later non-Linux decoder work, but not proof that a non-Linux path consumed it. `evidence/m2-video-path-validation/notes.md:54` through `evidence/m2-video-path-validation/notes.md:80` and `evidence/m2-video-path-validation/bundle-manifest.json:110` through `evidence/m2-video-path-validation/bundle-manifest.json:203` explicitly exclude direct capture DMA-BUF import into NVENC, zero-copy capture-to-encode, no-readback for the grim-based encode sample, HDR/color, latency, throughput, Mac decode/render, and cross-device behavior.

## Acceptance coverage notes

- PipeWire/XDPH discovery coverage is adequate for its intended result: it recorded versions, APIs, and the inability to select a named output without a consented stream or direct capture path. See `reports/qd/m2-pipewire-probe/completion.json:44` through `reports/qd/m2-pipewire-probe/completion.json:64` and `reports/qd/m2-pipewire-probe/blocker.md:15` through `reports/qd/m2-pipewire-probe/blocker.md:20`.
- Portal proof coverage is incremental and correctly bounded. Early portal nodes document `SelectSources` timeout or access-denied blockers (`reports/qd/m2-portal-screencast-client-proof/blocker.md:10` through `reports/qd/m2-portal-screencast-client-proof/blocker.md:23`, `reports/qd/m2-portal-chooser-interaction-proof/blocker.md:15` through `reports/qd/m2-portal-chooser-interaction-proof/blocker.md:31`). Manual-consent evidence later reaches stream metadata but does not consume a frame.
- Direct capture coverage is sufficient for the M2 capture metadata claim. `m2-direct-capture-api-spike` identified the path but did not implement a client (`reports/qd/m2-direct-capture-api-spike/completion.json:93` through `reports/qd/m2-direct-capture-api-spike/completion.json:98`); `m2-native-dmabuf-allocation-proof` and `m2-capture-one-frame` then closed the critical evidence gap with GBM allocation, DMA-BUF import, and `frame.ready=true`.
- The capture contract is platform-neutral and does not overclaim runtime behavior. `reports/qd/m2-capture-contract/completion.json:69` through `reports/qd/m2-capture-contract/completion.json:73` explicitly mark live validation as not required.
- The M2 Linux sample can be treated as a stable decoder-consumable input for M3 decode tests, with hash/size/ffprobe metadata. It must not be treated as evidence for live streaming, latency, Mac rendering, HDR/color correctness, zero-copy encode, or capture-to-NVENC no-readback.

## Recommended dispositions and follow-up nodes

- Disposition for M3: allow `evidence/m2-nvenc-encode-sample/sample-av1.ivf` as a stable one-frame AV1/IVF decoder input only. Require M3 validation nodes to produce their own decode/render/presentation evidence.
- Disposition for capture: accept direct `ext-image-copy-capture-v1` + GBM DMA-BUF evidence as sufficient to unblock narrow Linux capture-metadata consumers. Do not require the portal path for this narrow purpose.
- Follow-up node: clean stale `reports/qd/m2-capture-one-frame/blocker.md` or replace it with a superseded-blocker note referencing the successful GBM/direct-capture evidence.
- Follow-up node: add a repo-supported direct capture helper/dev-shell dependencies using `pkg-config`, `wayland-scanner`, `wayland`, `wayland-protocols`, `libgbm`, and `libdrm`; preserve evidence that it can rebuild from the repo shell.
- Follow-up node: direct capture-to-NVENC import proof, if any future node wants zero-copy/no-readback capture-to-encode claims.
- Follow-up node: end-to-end streaming performance and cross-device latency validation before any M3/M4 performance claim.
- Follow-up node: Mac VideoToolbox decode/render/presentation validation for `sample-av1.ivf`; this is required before treating the sample as proof of downstream Mac behavior.
- Follow-up node: HDR/color-management validation with tagged source material and measured/inspected output before HDR or wide-gamut claims.

## Commands run

- `pwd && git status --short --branch`
- `sed -n '1,240p' docs/ORCHESTRATION.md`
- `sed -n '241,520p' docs/ORCHESTRATION.md`
- `sed -n '1,260p' docs/LINUX_ORCHESTRATOR.md`
- `sed -n '1,260p' docs/CROSS_DEVICE_VALIDATION.md`
- `qd node show m2-audit --full` failed with `DB schema is older than this qd binary (no schema_migrations table found). Run qd migrate.` No migration was run.
- `find reports/qd evidence -maxdepth 3 -type f | sort | rg 'm2-(...)'`
- `rg -n "capture|PipeWire|portal|screencast|blocker|P0|P1|m2-" reports/qd evidence roadmap docs -g '*.md' -g '*.json'`
- `jq '.nodes[]? | select(.id=="m2-audit")' roadmap/qd-export.json`
- `jq '.nodes[]? | select(.id|test("^m2-(...)$")) | {id,status,dependencies,acceptance_criteria,findings,blockers}' roadmap/qd-export.json`
- Multiple `nl -b a` reads over the reviewed report/evidence files listed above.
- `stat -c '%y %n' reports/qd/m2-capture-one-frame/blocker.md reports/qd/m2-capture-one-frame/completion.json reports/qd/m2-capture-one-frame/audit.json evidence/m2-capture-one-frame/direct-capture-frame.json evidence/m2-capture-one-frame/validation-summary.json`
- `jq 'keys' roadmap/qd-export.json`
- `jq '.findings? // empty' roadmap/qd-export.json | head -200`
- `git status --short --branch`
