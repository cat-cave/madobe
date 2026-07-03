# m3-videotoolbox-capability-probe

This node records local macOS VideoToolbox/CoreMedia AV1 decode capability before
Linux M2 provides a captured encoded frame.

It does not claim a captured-frame decode, stream decode, PipeWire path, or
cross-device latency result. Those remain owned by the existing M2 and M3 nodes.

Local probe result:

- `kCMVideoCodecType_AV1` resolves to `av01`.
- `VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1)` returned `true`.
- `just macos-check` passed on the node worktree.
