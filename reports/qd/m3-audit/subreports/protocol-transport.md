# M3 Protocol/Transport Audit

## Scope

Owned subreport for protocol, transport, result compatibility, and cross-device smoke boundaries. Reviewed:

- `m3-video-protocol`
- `m3-swift-protocol-mirror`
- `m3-quic-transport-skeleton`
- `m3-cross-device-result-schema-fixture`
- `m3-lan-video-smoke-harness`
- `m3-cross-device-video-smoke`
- `crates/protocol` encoded video and cross-device result types/fixtures
- Swift protocol mirror tests
- `crates/transport` loopback skeleton and TCP LAN harness

This audit checks whether M3 claims are correctly bounded and whether any P0/P1 compatibility, transport, or result-schema blockers exist.

## Commands And Files Reviewed

Focused commands run:

- `rg -n "m3-|M3|cross-device|result schema|LAN|QUIC|video smoke|EncodedVideo|smoke" -S crates reports docs .qd`
- `jq empty` on reviewed completion/result/fixture JSON files
- `cargo test -p madobe-protocol cross_device -- --nocapture` direct attempt failed because `cargo` was not on `PATH`
- `nix develop -c cargo test -p madobe-protocol cross_device -- --nocapture` passed: 5 tests
- `nix develop -c cargo test -p madobe-transport --all-features` passed: 7 tests plus doc-tests

Key files reviewed:

- `crates/protocol/src/lib.rs`
- `crates/protocol/src/cross_device.rs`
- `crates/protocol/fixtures/encoded-video-frame-av1.json`
- `crates/protocol/fixtures/m3-cross-device-video-smoke/result.json`
- `apple/Sources/MadobeClientCore/EncodedVideoFrameMetadata.swift`
- `apple/Tests/MadobeClientCoreTests/MadobeClientCoreTests.swift`
- `crates/transport/src/lib.rs`
- `crates/transport/src/video_smoke/{mod.rs,wire.rs,cli.rs,artifacts.rs,sha256.rs}`
- `crates/transport/tests/{loopback.rs,lan_video_smoke.rs}`
- `docs/protocol/encoded-video-frame.md`
- `docs/LAN_VIDEO_SMOKE_HARNESS.md`
- `docs/CROSS_DEVICE_VALIDATION.md`
- `reports/qd/m3-*/completion.json` for the scoped nodes
- `evidence/m3-cross-device-video-smoke/result.json`
- `evidence/m3-cross-device-video-smoke/macos/linux-attempt/result.json`
- `evidence/m3-lan-video-smoke-harness/result.json`

## Findings By Severity

### P0

None.

### P1

None.

### P2/P3 Follow-ups

- Product QUIC is not implemented or validated cross-device in M3. `madobe-transport` is explicitly an in-memory QUIC-shaped skeleton, and the live LAN smoke uses the TCP helper.
- The cross-device smoke proves one Linux-to-Mac TCP transfer of the checked-in 84-byte AV1 IVF sample with matching metadata, byte count, and SHA-256. It does not prove live capture, product transport, VideoToolbox decode, Metal render, presentation, or latency.
- The result guardrail validates metric/evidence role consistency, but artifact role presence is not the same as semantic proof. Future nodes should attach stronger machine-checked validation for decode/render/presentation/latency artifacts before nonzero metrics are accepted as product evidence.
- The protocol result fixture is a Rust-typed/manual golden fixture, not a standalone JSON Schema contract. If external producers expand, add serde or JSON Schema validation instead of relying only on golden-string tests.

## Acceptance Evidence

- Encoded video protocol is bounded to metadata. `EncodedVideoFrameMetadata` documents that it does not claim capture, transport, decode, render, or latency behavior, and Rust golden tests cover frame id, codec, dimensions, timestamps, keyframe flag, payload bytes, and SHA-256.
- Swift protocol mirror decodes the Rust-authored golden fixture through `JSONDecoder`, asserts semantic equality, rejects an unknown codec, and confirms payload hash mismatch changes semantics.
- Cross-device result schema fixture has required fields, nullable latency metrics, and guardrail tests rejecting decode/render/presentation/latency metrics without matching artifact kinds. Focused local run passed all 5 `cross_device` tests.
- Transport skeleton claims are correctly scoped. `LoopbackQuicEndpoint` and `VideoLane` are in-memory only, tests cover sample exchange, telemetry timestamps, typed payload-size errors, and stable video lane id.
- LAN harness claims are correctly scoped. The TCP helper sends the checked-in AV1 IVF sample as opaque bytes, validates AV1/160x90/frame id/keyframe/hash metadata, byte count `84`, and SHA-256 `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`. Focused `madobe-transport` tests passed.
- Cross-device video smoke evidence is bounded correctly. `evidence/m3-cross-device-video-smoke/result.json` records `passed=true`, `frames_sent=1`, `frames_decoded=0`, `frames_rendered=0`, `frames_presented=0`, and null latency metrics, with notes excluding product QUIC, VideoToolbox decode, Metal render, presentation, and latency.

## Recommendation

Pass for M3 protocol/transport/result compatibility as scoped. No P0/P1 audit findings. Do not treat M3 as product QUIC, decode, render, presentation, or latency validation; keep those as explicit follow-up validation nodes before making broader product claims.
