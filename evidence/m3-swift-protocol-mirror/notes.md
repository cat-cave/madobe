# m3-swift-protocol-mirror Notes

This node mirrors the Rust-authored encoded video frame fixture on the Swift client side.

Acceptance mapping:

- Swift tests read or mirror the golden fixture semantics:
  `MadobeClientCoreTests.testEncodedVideoFrameFixtureMatchesRustGoldenVector` reads
  `crates/protocol/fixtures/encoded-video-frame-av1.json` from the repository and decodes it into
  `EncodedVideoFrameMetadata`.
- Tuist/Xcode tests pass on macOS: `evidence/m3-swift-protocol-mirror/xcodebuild.log` and
  `evidence/m3-swift-protocol-mirror/just-macos-check.log`.
- Fixture mismatch behavior is tested:
  `testEncodedVideoFrameRejectsUnknownCodecFixtureMismatch` and
  `testEncodedVideoFramePayloadHashMismatchChangesSemantics`.
- Mac evidence includes xcodebuild log: `evidence/m3-swift-protocol-mirror/xcodebuild.log`.

The Swift model uses `UInt64` for frame and timestamp fields, `UInt32` for dimensions and payload byte count,
string-backed enums for `av1` and `sha256`, and preserves the payload hash value exactly.
