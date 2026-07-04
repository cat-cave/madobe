# m3-videotoolbox-decode-sample Notes

This node verifies that the checked-in Linux AV1 IVF sample from `evidence/m2-nvenc-encode-sample/sample-av1.ivf` can be parsed and decoded by VideoToolbox on this Mac.

Evidence summary:

- Input SHA-256: `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`.
- Parsed container: IVF with AV01 fourcc, one 160x90 frame, timescale 60.
- Frame payload bytes: 40.
- VideoToolbox sample payload bytes after stripping the leading temporal delimiter OBU: 38.
- `av1C`: `81000c000a0a00000003b4fd90086601`.
- `VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1)`: true on this Mac.
- Xcode test-host decode result: decoded 160x90 `420v` frame with hardware-accelerated decoder true.
- Decode timing is in `decode-report.json`.

Validation path:

- `xcodebuild.log` is the focused Xcode test-host validation that generated `decode-report.json`.
- The final qd gate uses `just macos-check`; its log is saved as `macos-check.log` after the full check run.

Important limits:

- This does not claim Metal rendering, cross-device latency, live network transport, HDR/color-management validation, streaming decode performance, or any Linux capture-path property beyond consuming the checked-in M2 AV1 sample.
- A separately compiled unsigned Swift CLI helper returned `VTDecompressionSessionCreate -12911` on this host. The signed Xcode test host and the interpreted Swift probe both decode successfully, so the node evidence uses the Xcode test-host path that matches the macOS app/test execution environment.
