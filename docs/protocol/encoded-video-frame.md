# Encoded Video Frame Metadata

`m3-video-protocol` defines the first shared metadata fixture for encoded video frames. This is a protocol fixture
only; it does not claim live capture, QUIC transport, VideoToolbox decode, Metal render, or latency behavior.

Golden fixture:

- `crates/protocol/fixtures/encoded-video-frame-av1.json`

Rust type:

- `madobe_protocol::EncodedVideoFrameMetadata`

## JSON Shape

The wire JSON uses lower camel case field names:

- `frameId`: unsigned 64-bit sender-local frame id.
- `codec`: string enum. Current fixture value is `av1`.
- `width`: unsigned 32-bit pixel width.
- `height`: unsigned 32-bit pixel height.
- `captureTimestampNs`: unsigned 64-bit sender-local capture timestamp in nanoseconds.
- `encodeTimestampNs`: unsigned 64-bit sender-local encode completion timestamp in nanoseconds.
- `keyframe`: boolean decoder-state initialization marker.
- `payloadBytes`: unsigned 32-bit encoded payload byte count.
- `payloadHash.algorithm`: string enum. Current fixture value is `sha256`.
- `payloadHash.value`: 64-character lowercase SHA-256 hex digest.

## Swift Mirror Requirements

The Swift mirror should preserve the JSON field names and scalar widths:

- `frameId`, `captureTimestampNs`, and `encodeTimestampNs` map to `UInt64`.
- `width`, `height`, and `payloadBytes` map to `UInt32` or a checked wider integer with range validation.
- `codec` should be a string-backed enum with at least `av1`.
- `payloadHash.algorithm` should be a string-backed enum with at least `sha256`.
- `payloadHash.value` should be preserved exactly as lowercase hex.
- Fixture mismatch behavior should be tested by the Swift node rather than silently defaulting fields.

The Swift node should consume `crates/protocol/fixtures/encoded-video-frame-av1.json` or a byte-identical copy and
compare decoded values to the Rust fixture expectations.
