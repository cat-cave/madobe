# m3-cross-device-video-smoke

This node coordinates the first live two-machine video smoke between the Linux host and Mac client.

The first live attempt uses the dependency-free TCP LAN harness from `m3-lan-video-smoke-harness`:

- Mac runs `madobectl video-smoke receive` and records receiver evidence.
- Linux runs `madobectl video-smoke send` with the checked-in AV1 IVF sample and records sender evidence.
- The checked sample hash is `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`.

Expected claim boundary for this first attempt:

- It can prove cross-device connectivity and sample byte transfer.
- It can record frames sent and a Mac receiver log.
- It does not by itself prove product QUIC, live capture, VideoToolbox decode, Metal render, presentation, or latency.

If decode/render/presentation cannot be driven from the received sample in the current app/tooling, the final result must record zero for those metrics and create qd findings or follow-up nodes instead of implying success.

Live run status:

- The first Mac receiver result at commit `633ae25` validated the sample on
  the receiver path, but it was explicitly Mac-local and is not Linux-to-Mac
  proof.
- Linux later ran the real sender from this workstation to
  `192.168.1.15:47044` using the checked-in AV1 sample.
- Linux sender artifacts under `linux/` prove connection and send from the
  Linux worktree; final completion still requires the Mac receiver result for
  that Linux-originated send.
- Mac receiver artifacts under `macos/linux-attempt/` record the matching
  receiver result for the Linux-originated send. The receiver observed peer
  `192.168.1.23:46968`, payload bytes `84`, and the expected SHA-256.
- The completed live attempt proves TCP LAN sample transfer and byte/hash
  validation only. It does not prove product QUIC, VideoToolbox decode, Metal
  render, presentation, or latency.
- Top-level `result.json`, `linux-host.log`, and `mac-client.log` aggregate the
  side-specific evidence in the shape required by `docs/CROSS_DEVICE_VALIDATION.md`.
  The aggregate records `frames_sent: 1`; decode, render, and presentation remain
  `0`, and latency metrics remain `null`.
