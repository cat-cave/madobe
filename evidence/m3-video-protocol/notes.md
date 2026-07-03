# m3-video-protocol Notes

This node adds the first encoded video frame metadata fixture and Rust wire type. It is intentionally limited to
metadata and golden-vector behavior; it does not claim live capture, QUIC transport, decode, render, or latency.

Acceptance mapping:

- Rust protocol types compile under workspace lints: `just check`.
- Golden fixtures cover frame id, timestamps, codec, dimensions, keyframe flag, and payload hash:
  `crates/protocol/fixtures/encoded-video-frame-av1.json`.
- Swift mirror requirements are documented: `docs/protocol/encoded-video-frame.md`.
- Round-trip or golden-vector tests exist: `madobe-protocol` tests decode the fixture and round-trip it through
  `serde_json::Value`.

The payload hash fixture value is SHA-256 of `madobe-m3-golden-video-frame-payload-v1`.

Dependency note: the final implementation adds no external Rust dependencies. An intermediate serde/serde_json approach
was removed after Linux CI showed it would require new cargo-vet coverage beyond this protocol fixture node.
