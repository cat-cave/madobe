# M2 Downstream Readiness Audit

Node: `m2-audit`

Scope: downstream sample expectations and M3-readiness/non-claims before `m3-cross-device-video-smoke`.

## Reviewed files and evidence

- Required runbooks: `docs/ORCHESTRATION.md`, `docs/LINUX_ORCHESTRATOR.md`, `docs/CROSS_DEVICE_VALIDATION.md`.
- Read-only qd fallback: `roadmap/qd-export.json` entries for `m2-audit`, `m3-cross-device-video-smoke`, `m3-videotoolbox-decode-sample`, `m3-metal-renderer-skeleton`, `m3-linux-video-smoke-host-preflight`, and `m3-lan-video-smoke-harness`. `qd node show m2-audit --full` failed because the local qd DB schema needs migration; no migration was run.
- M2 bundle/report: `evidence/m2-video-path-validation/{bundle-manifest.json,notes.md}`, `reports/qd/m2-video-path-validation/{completion.json,audit.json}`.
- M2 audit-fix reports: `evidence/m2-video-path-validation-does-not-*/report.md` and matching `reports/qd/m2-video-path-validation-does-not-*/completion.json`.
- M3 protocol/schema fixtures: `crates/protocol/fixtures/encoded-video-frame-av1.json`, `docs/protocol/encoded-video-frame.md`, `crates/protocol/fixtures/m3-cross-device-video-smoke/result.json`, `crates/protocol/src/cross_device.rs`, `evidence/m3-cross-device-result-schema-fixture/*`, and `reports/qd/m3-cross-device-result-schema-fixture/*`.
- M3 sample consumers/preflight/harness: `evidence/m3-videotoolbox-decode-sample/*`, `evidence/m3-metal-renderer-skeleton/*`, `docs/LINUX_VIDEO_SMOKE_HOST_PREFLIGHT.md`, `evidence/m3-linux-video-smoke-host-preflight/*`, `docs/LAN_VIDEO_SMOKE_HARNESS.md`, `evidence/m3-lan-video-smoke-harness/*`, and matching qd reports.

## Findings by severity

### P0

None.

### P1

None. The downstream sample expectations are documented clearly enough to prevent treating the M2 artifact as proof of live Mac decode/render/presentation/latency behavior.

### P2

- `m3-linux-video-smoke-host-preflight` is useful sample/source preflight evidence, but it is not proof that the Linux live display lifecycle will be available during cross-device validation. The evidence says `HYPRLAND_INSTANCE_SIGNATURE` was unset and the live Hyprland instance was unreachable (`evidence/m3-linux-video-smoke-host-preflight/notes.md:15` through `evidence/m3-linux-video-smoke-host-preflight/notes.md:24`; `evidence/m3-linux-video-smoke-host-preflight/host-preflight-summary.json:20` through `evidence/m3-linux-video-smoke-host-preflight/host-preflight-summary.json:24`). Disposition: acceptable if `m3-cross-device-video-smoke` only sends the checked-in sample, but the live smoke must rerun and record host command/log evidence on the actual synced branch before claiming host display readiness.
- `m3-metal-renderer-skeleton` has clear local render evidence, but its `render-report.json` field `presentedTestPattern=true` can be misread as display presentation evidence. The report is produced by an offscreen Metal texture render/sampling path (`apple/Sources/MadobeClientCore/MetalRenderTimingProbe.swift:111` through `apple/Sources/MadobeClientCore/MetalRenderTimingProbe.swift:137`), while the separate app view path is what calls `commandBuffer.present(drawable)` (`apple/Sources/MadobeMac/MetalTestPatternView.swift:38` through `apple/Sources/MadobeMac/MetalTestPatternView.swift:56`). Disposition: for `m3-cross-device-video-smoke`, treat this as `render_evidence` only. Do not set nonzero `frames_presented` unless the result includes a dedicated `presentation_evidence` artifact such as app/display logs or screenshot evidence from the live Mac client.

### P3

- The cross-device result validator checks that nonzero decode/render/presentation/latency metrics have matching artifact roles, but it does not validate artifact existence or artifact content (`crates/protocol/src/cross_device.rs:152` through `crates/protocol/src/cross_device.rs:199`). Disposition: acceptable for the schema-fixture node; the live `m3-cross-device-video-smoke` audit must inspect the referenced artifacts directly.

## Acceptance coverage notes

- The M2 bundle names the downstream sample as `evidence/m2-nvenc-encode-sample/sample-av1.ivf` and records AV1/IVF metadata, one frame, 84 bytes, and SHA-256 `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875` (`evidence/m2-video-path-validation/bundle-manifest.json:62` through `evidence/m2-video-path-validation/bundle-manifest.json:88`; `evidence/m2-video-path-validation/bundle-manifest.json:231` through `evidence/m2-video-path-validation/bundle-manifest.json:234`).
- The M2 evidence boundary is machine-checkable and conservative. It marks direct DMA-BUF import into NVENC, zero-copy/no-readback encode, HDR/color behavior, streaming/network/latency, downstream Mac decode/render, VideoToolbox, Metal render, Mac presentation, and Mac timing claims as false or not-evidence (`evidence/m2-video-path-validation/bundle-manifest.json:118` through `evidence/m2-video-path-validation/bundle-manifest.json:141`; `evidence/m2-video-path-validation/bundle-manifest.json:177` through `evidence/m2-video-path-validation/bundle-manifest.json:203`; `evidence/m2-video-path-validation/bundle-manifest.json:303` through `evidence/m2-video-path-validation/bundle-manifest.json:325`).
- The follow-up M2 audit-fix nodes preserve those boundaries: downstream Mac decode/render remains future proof in the M2 bundle (`evidence/m2-video-path-validation-does-not-prove-downstream-mac-decode-re/report.md:9` through `evidence/m2-video-path-validation-does-not-prove-downstream-mac-decode-re/report.md:31`), streaming latency/performance remains unmeasured (`evidence/m2-video-path-validation-does-not-measure-streaming-latency-or-p/report.md:11` through `evidence/m2-video-path-validation-does-not-measure-streaming-latency-or-p/report.md:33`), and the grim/PNG/RGB encode path remains non-zero-copy/non-no-readback evidence (`evidence/m2-video-path-validation-does-not-prove-direct-dma-buf-import-in/report.md:11` through `evidence/m2-video-path-validation-does-not-prove-direct-dma-buf-import-in/report.md:27`).
- M3 now has real Mac VideoToolbox decode evidence for the checked-in M2 sample: `decode-report.json` records the same sample path/hash, decoded status, 160x90 output, `420v`, and hardware-accelerated decode (`evidence/m3-videotoolbox-decode-sample/decode-report.json:5` through `evidence/m3-videotoolbox-decode-sample/decode-report.json:19`). Its notes still exclude Metal render, cross-device latency, live network transport, HDR/color, streaming decode performance, and Linux capture-path claims (`evidence/m3-videotoolbox-decode-sample/notes.md:21` through `evidence/m3-videotoolbox-decode-sample/notes.md:24`).
- M3 schema fixtures align with `docs/CROSS_DEVICE_VALIDATION.md`: zero decoded/rendered/presented frames, null latency, and notes that the fixture is schema-only (`crates/protocol/fixtures/m3-cross-device-video-smoke/result.json:9` through `crates/protocol/fixtures/m3-cross-device-video-smoke/result.json:27`; `docs/CROSS_DEVICE_VALIDATION.md:144` through `docs/CROSS_DEVICE_VALIDATION.md:151`).
- The LAN harness is suitable for cross-device sample transport preparation only. It sends the checked-in sample as opaque TCP payload, validates metadata, byte count, and SHA-256, and explicitly excludes QUIC/product transport, VideoToolbox decode, Metal render, presentation, and latency proof (`docs/LAN_VIDEO_SMOKE_HARNESS.md:3` through `docs/LAN_VIDEO_SMOKE_HARNESS.md:16`; `docs/LAN_VIDEO_SMOKE_HARNESS.md:52` through `docs/LAN_VIDEO_SMOKE_HARNESS.md:64`; `evidence/m3-lan-video-smoke-harness/result.json:1` through `evidence/m3-lan-video-smoke-harness/result.json:22`).

## Recommended dispositions and follow-up nodes

- Allow the M2 `sample-av1.ivf` as the stable M3 sample input. It is a one-frame AV1/IVF decoder input with recorded hash/size, not a performance, HDR/color, zero-copy, no-readback, or cross-device proof.
- For `m3-cross-device-video-smoke`, require `result.json` to record only the metrics actually proven by artifacts from that live run. Use `decode_evidence`, `render_evidence`, `presentation_evidence`, and `latency_evidence` artifact kinds only when the corresponding logs/screenshots/reports exist and are inspected.
- Before claiming Linux live host readiness, rerun the host command/log capture from a shell that reaches the live Hyprland instance, or explicitly scope the smoke to checked-in sample send/receive without live display lifecycle.
- Follow-up node if presentation will be claimed: add Mac client presentation evidence for the decoded/sample frame, ideally with app/display logs and screenshot or equivalent visible-output artifact, then use that artifact as `presentation_evidence`.
- Follow-up node if product transport will be claimed: run QUIC/product transport cross-device validation; the current LAN harness is intentionally TCP-only and non-product.
- Follow-up node if latency will be claimed: add a latency-specific cross-device node with timeline artifacts and non-null latency metrics.

## Commands run

- `sed -n '1,240p' docs/ORCHESTRATION.md`
- `sed -n '241,520p' docs/ORCHESTRATION.md`
- `sed -n '1,260p' docs/LINUX_ORCHESTRATOR.md`
- `sed -n '1,260p' docs/CROSS_DEVICE_VALIDATION.md`
- `qd node show m2-audit --full` failed with `DB schema is older than this qd binary (no schema_migrations table found). Run qd migrate.` No migration was run.
- `git status --short --branch`
- `jq '.nodes[] | select(.id=="m2-audit")' roadmap/qd-export.json`
- `jq '.nodes[] | select(.id=="m3-cross-device-video-smoke" or .id=="m3-linux-video-smoke-host-preflight" or .id=="m3-lan-video-smoke-harness" or .id=="m3-videotoolbox-decode-sample" or .id=="m3-metal-renderer-skeleton") | {id,title,status,priority,branch,spec,acceptance,blocked_by,blocked_reason}' roadmap/qd-export.json`
- `rg --files reports/qd evidence docs crates | rg '(^reports/qd/m2-video-path-validation|^evidence/m2-video-path-validation|^reports/qd/m2-video-path-validation-does-not|^evidence/m2-video-path-validation-does-not|^reports/qd/m3-|^evidence/m3-|^crates/.*/fixtures/m3|CROSS_DEVICE|ORCHESTRATION|LINUX_ORCHESTRATOR)'`
- Multiple `nl -b a ... | sed -n ...` reads over the reviewed files listed above.
- `rg -n 'VideoToolbox decode|Metal render|Mac presentation|presentation evidence|latency proof|latency claim|decoder-consumable|sample-av1|frames_decoded|frames_rendered|frames_presented|median_glass|p95_glass|not QUIC|product transport|cross-device' ...`
- `jq -e . evidence/m2-video-path-validation/bundle-manifest.json reports/qd/m2-video-path-validation/completion.json reports/qd/m2-video-path-validation/audit.json crates/protocol/fixtures/m3-cross-device-video-smoke/result.json evidence/m3-linux-video-smoke-host-preflight/host-preflight-summary.json evidence/m3-lan-video-smoke-harness/result.json reports/qd/m3-cross-device-result-schema-fixture/completion.json reports/qd/m3-linux-video-smoke-host-preflight/completion.json reports/qd/m3-lan-video-smoke-harness/completion.json`
- `sha256sum evidence/m2-nvenc-encode-sample/sample-av1.ivf`
- `stat -c '%s %n' evidence/m2-nvenc-encode-sample/sample-av1.ivf`
- `rg -n 'directCaptureDmaBufImportIntoNvencProven|zeroCopyCaptureToEncodeProven|noCpuReadbackAfterGrimCaptureProven|downstreamMacDecodeRenderValidated|videoToolboxDecodeProven|metalRenderProven|macPresentationDisplayValidated|crossDeviceLatencyMeasured|decoderConsumableAv1IvfSampleProven|explicitNonClaims|not QUIC|not latency proof|Schema fixture only|HYPRLAND_INSTANCE_SIGNATURE|presentedTestPattern' ...`
- `git diff --check -- reports/qd/m2-audit/subreports/downstream-readiness.md`
- `git status --short`
