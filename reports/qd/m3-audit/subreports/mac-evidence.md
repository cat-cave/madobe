# Mac-Side M3 Evidence Audit

Audit date: 2026-07-04
Owned scope: `reports/qd/m3-audit/subreports/mac-evidence.md`

## Scope

Reviewed Mac-side M3 evidence for:

- Swift fixture compatibility and encoded-video metadata parity.
- VideoToolbox AV1 capability and checked-in IVF sample decode evidence.
- Metal renderer skeleton and local render timing evidence.
- Mac receive queue validation and timeline evidence.
- Mac receiver artifacts for `m3-cross-device-video-smoke`, including the corrected Linux-originated attempt.

This audit did not mutate qd state, commit changes, or edit any file outside this subreport.

## Commands And Files Reviewed

Focused read/check commands:

- `git status --short --branch`
- `rg --files apple evidence reports/qd`
- `find evidence/m3-videotoolbox-capability-probe evidence/m3-videotoolbox-decode-sample evidence/m3-metal-renderer-skeleton evidence/m3-mac-video-receive-queue evidence/m3-cross-device-video-smoke -maxdepth 3 -type f`
- `find reports/qd -maxdepth 2 -type f \( -path 'reports/qd/m3-*/completion.json' -o -path 'reports/qd/m3-*/audit.json' \)`
- `jq .` over relevant M3 `completion.json`, `audit.json`, and evidence JSON files.
- `sed`/`tail` over relevant Swift sources, tests, notes, command logs, receiver/sender logs, and xcodebuild excerpts.
- `rg -n` for claim-boundary terms such as decode, render, present, latency, QUIC, finding, failed, and test success markers.

Key files reviewed:

- `apple/Sources/MadobeClientCore/EncodedVideoFrameMetadata.swift`
- `apple/Sources/MadobeClientCore/VideoToolboxCapabilityProbe.swift`
- `apple/Sources/MadobeClientCore/VideoToolboxAV1SampleDecoder.swift`
- `apple/Sources/MadobeClientCore/VideoToolboxAV1DecodeCore.swift`
- `apple/Sources/MadobeClientCore/VideoReceiveQueue.swift`
- `apple/Sources/MadobeClientCore/MetalRenderTimingProbe.swift`
- `apple/Sources/MadobeMac/MetalTestPatternView.swift`
- `apple/Tests/MadobeClientCoreTests/MadobeClientCoreTests.swift`
- `apple/Tests/MadobeClientCoreTests/VideoToolboxCapabilityProbeTests.swift`
- `apple/Tests/MadobeClientCoreTests/VideoToolboxAV1SampleDecoderTests.swift`
- `apple/Tests/MadobeClientCoreTests/VideoReceiveQueueTests.swift`
- `apple/Tests/MadobeClientCoreTests/MetalRenderTimingProbeTests.swift`
- `crates/protocol/fixtures/encoded-video-frame-av1.json`
- `crates/protocol/fixtures/m3-cross-device-video-smoke/result.json`
- `crates/protocol/src/cross_device.rs`
- Evidence and reports under `evidence/m3-videotoolbox-*`, `evidence/m3-metal-renderer-skeleton`, `evidence/m3-mac-video-receive-queue`, `evidence/m3-cross-device-video-smoke`, and `reports/qd/m3-*`.

## Findings By Severity

### P0

None.

### P1

None.

### P2

- Metal evidence should keep the word "presented" tightly bounded. `evidence/m3-metal-renderer-skeleton/render-report.json` records `presentedTestPattern: true`, but the probed path is an offscreen BGRA8 render target sampled by CPU readback. The separate app shell path does wire an `MTKView` in `MetalTestPatternView.swift`, and `notes.md` correctly says no GUI screenshot was captured. This is not a blocker because no streaming, decoded-frame, cross-device, or latency claim is made, but future reports should avoid implying onscreen presentation unless a drawable/screenshot/display artifact is captured.

### P3

- The cross-device smoke keeps product QUIC, live received-sample VideoToolbox decode, Metal render/presentation, and latency as explicit follow-up work. This is correctly bounded today, but those items remain required before broader product-path or glass-to-glass claims.
- The receive queue is intentionally synthetic/in-memory. It validates byte count, SHA-256, depth-one stale-frame behavior, and Codable timelines, but it is not yet connected to the TCP/QUIC receiver, VideoToolbox decode, or Metal render path.

## Acceptance Evidence

Swift fixture compatibility:

- `EncodedVideoFrameMetadata.swift` matches the Rust-authored camelCase fixture fields for AV1 metadata, including `frameId`, `codec`, timestamps, `payloadBytes`, and nested SHA-256 hash fields.
- `MadobeClientCoreTests.swift` decodes `crates/protocol/fixtures/encoded-video-frame-av1.json`, checks the full expected semantic payload, rejects unknown codec fixture mutation, and detects payload-hash semantic mismatch.
- `reports/qd/m3-swift-protocol-mirror/audit.json` records no findings.

VideoToolbox capability and decode:

- `capability-report.json` records `coreMediaFourCharacterCode: "av01"` and `hardwareDecodeSupported: true` on macOS 26.5.1 / Mac17,2.
- `decode-report.json` records the checked-in IVF sample hash `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`, one 160x90 AV1 frame, `decodeStatus: "decoded"`, `decodedPixelFormat: "420v"`, `usingHardwareAcceleratedDecoder: true`, and non-null decode timing.
- `xcodebuild.log` and `macos-check.log` show passing Xcode/macOS gates for the relevant tests. Notes correctly bound the unsigned standalone CLI failure as non-blocking and use the signed Xcode test host as the accepted app-like environment.

Metal renderer:

- `MetalTestPatternView.swift` wires an `MTKView` renderer into the app shell.
- `MetalRenderTimingProbe.swift` creates a Metal device, command queue, BGRA8 render target, clear pass, command buffer, completion check, and pixel sample.
- `render-report.json` records device `Apple M5`, `commandBufferStatus: "completed"`, sampled BGRA8 `[0, 0, 255, 255]`, and non-null render timing.
- `notes.md` explicitly excludes live streaming, cross-device validation, end-to-end latency, decoded-frame scheduling, HDR/color management, and Linux host behavior.

Mac receive queue:

- `VideoReceiveQueue.swift` validates payload byte count and SHA-256 hash before accepting frames, keeps a single ready frame, emits dropped/received/dequeued events, and exposes stable Codable timeline output.
- `VideoReceiveQueueTests.swift` covers valid enqueue/dequeue, byte-count mismatch, hash mismatch, stale-frame drop, and stable JSON.
- `client-receive-timeline.json` records deterministic stale-drop evidence for frame 4 replaced by frame 5.
- `reports/qd/m3-mac-video-receive-queue/audit.json` records no findings and correctly states no live network, capture, decode, render, or latency claim.

Cross-device Mac receiver artifacts:

- The initial `macos/result.json` and `receiver-notes.md` are correctly marked as Mac-local because the observed peer was `192.168.1.15`.
- The corrected `macos/linux-attempt/result.json`, `receiver.log`, and `receiver-timeline.json` record bind `0.0.0.0:47044`, peer `192.168.1.23:46968`, one received AV1 sample, `payloadBytes: 84`, SHA-256 `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`, and metadata/byte-count/hash validation success.
- Aggregate `evidence/m3-cross-device-video-smoke/result.json` records `frames_sent: 1` and keeps `frames_decoded`, `frames_rendered`, and `frames_presented` at `0`, with latency metrics `null`.
- `reports/qd/m3-cross-device-video-smoke/completion.json` lists follow-up DAG work for product QUIC, received-sample decode, render/presentation, and latency before any broader claim.

## Recommendation

Pass for Mac-side M3 evidence as currently claimed.

No P0/P1 findings were found. The evidence supports local Swift fixture compatibility, local VideoToolbox capability/decode of the checked-in sample, local Metal render/timing skeleton evidence, synthetic receive-queue behavior, and one bounded Linux-to-Mac TCP sample transfer. It does not support product QUIC, cross-device decode, cross-device render/presentation, or latency claims, and the reports correctly avoid claiming those.
